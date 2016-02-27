use time::SteadyTime;

use mio::{self, EventLoop, Token, EventSet, Sender, Timeout};
use mio::util::Slab;
use void::{Void, unreachable};

use scope::scope;
use {SpawnError, Scope, Response, Machine, Time, GenericScope};
use SpawnError::{NoSlabSpace};
use loop_time::{make_time, diff_ms};
use response::{decompose};


#[doc(hidden)]
pub enum Timeo {
    Fsm(Token),
}

#[doc(hidden)]
pub enum Notify {
    Fsm(Token),
}


/// Standard mio loop handler
///
///
/// # Examples
///
/// ```ignore
/// extern crate mio;
/// extern crate rotor;
///
/// let mut event_loop = mio::EventLoop::new().unwrap();
/// let mut handler = rotor::Handler::new(Context, &mut event_loop);
/// let conn = handler.add_machine_with(&mut event_loop, |scope| {
///     Ok(StateMachineConstuctor(..))
/// });
/// assert!(conn.is_ok());
/// event_loop.run(&mut handler).unwrap();
/// ```
pub struct Handler<M: Machine>
{
    slab: Slab<(Option<(Timeout, Time)>, M)>,
    context: M::Context,
    channel: Sender<Notify>,
    start_time: SteadyTime,
}

pub fn create_handler<M: Machine>(slab: Slab<(Option<(Timeout, Time)>, M)>,
    context: M::Context, channel: Sender<Notify>)
    -> Handler<M>
{
    Handler {
        slab: slab,
        context: context,
        channel: channel,
        start_time: SteadyTime::now(),
    }
}
pub fn set_timeout_opt<S: GenericScope>(option: Option<Time>, scope: &mut S)
    -> Option<(Timeout, Time)>
{
    option.map(|new_ts| {
        let ms = diff_ms(scope.now(), new_ts);
        let tok = scope.timeout_ms(ms)
            .expect("Can't insert a timeout. You need to \
                     increase the timer capacity");
        (tok, new_ts)
    })
}

fn replacer<C, M, N>(token: Token,
    resp: Response<M, N>, old_timeo: Option<(Timeout, Time)>,
    scope: &mut Scope<C>, creator: &mut Option<N>)
    -> Option<(Option<(Timeout, Time)>, M)>
{
    let (mach, new, newtime) = decompose(token, resp);
    let rtime = if newtime != old_timeo.map(|(_, x)| x) {
        if let Some((tok, _)) = old_timeo {
            scope.clear_timeout(tok);
        }
        set_timeout_opt(newtime, scope)
    } else {
        old_timeo
    };
    *creator = new;
    mach.map(|m| (rtime, m)).ok() // the error is already logged in decompose()
}

fn machine_loop<M, F>(handler: &mut Handler<M>,
    eloop: &mut EventLoop<Handler<M>>, token: Token, fun: F)
    where M: Machine,
          F: FnOnce(M, &mut Scope<M::Context>) -> Response<M, M::Seed>
{
    let time = handler.loop_time();
    let ref mut context = handler.context;
    let ref mut channel = handler.channel;
    let mut creator = None;
    {
        let ref mut scope = scope(time, token, context, channel, eloop);
        handler.slab.replace_with(token, |(timeo, m)| {
            replacer(token, fun(m, scope), timeo, scope, &mut creator)
        }).ok();  // Spurious events are ok in mio
    }
    while let Some(new) = creator.take() {
        let mut new = Some(new);
        let ins = handler.slab.insert_with(|token| {
            let ref mut scope = scope(time, token, context, channel, eloop);
            let (mach, newm, newtime) = decompose(token,
                M::create(new.take().unwrap(), scope));
            newm.map(|x| unreachable(x));
            let m = mach.expect("You can't return Response::done() \
                  from Machine::create() until new release of slab crate. \
                  (requires insert_with_opt)");
            let timepair = newtime.map(|new_ts| {
                let ms = diff_ms(scope.now(), new_ts);
                let tok = scope.timeout_ms(ms)
                    .expect("Can't insert a timeout. You need to \
                             increase the timer capacity");
                (tok, new_ts)
            });
            (timepair, m)
        });
        if ins.is_none() {
            // TODO(tailhook) process other errors here, when they can
            // be returned from handler
            let err = NoSlabSpace(new.expect("expecting seed is still here"));

            let ref mut scope = scope(time, token, context, channel, eloop);
            handler.slab.replace_with(token, |(timeo, m)| {
                replacer(token, m.spawn_error(scope, err),
                    timeo, scope, &mut creator)
            }).ok();
        } else {
            let ref mut scope = scope(time, token, context, channel, eloop);
            handler.slab.replace_with(token, |(timeo, m)| {
                replacer(token, m.spawned(scope), timeo, scope, &mut creator)
            }).ok();
        }
    }
}

impl<M: Machine> Handler<M>
{
    pub fn loop_time(&self) -> Time {
        let now = SteadyTime::now();
        return make_time(self.start_time, now);
    }
    pub fn add_machine_with<F>(&mut self, eloop: &mut EventLoop<Self>, fun: F)
        -> Result<(), SpawnError<()>>
        where F: FnOnce(&mut Scope<M::Context>) -> Response<M, Void>
    {
        let time = self.loop_time();
        let ref mut context = self.context;
        let ref mut channel = self.channel;
        let res = self.slab.insert_with(|token| {
            let ref mut scope = scope(time, token, context, channel, eloop);
            let (mach, void, timeout) =  decompose(token, fun(scope));
            void.map(|x| unreachable(x));
            let m = mach.expect("You can't return Response::done() \
                  from Machine::create() until new release of slab crate. \
                  (requires insert_with_opt)");
            let to = set_timeout_opt(timeout, scope);
            (to, m)
        });
        if res.is_some() {
            Ok(())
        } else {
            // TODO(tailhook) propagate error from state machine construtor
            Err(NoSlabSpace(()))
        }
    }
}

impl<M: Machine> mio::Handler for Handler<M>
{
    type Message = Notify;
    type Timeout = Timeo;
    fn ready<'x>(&mut self, eloop: &'x mut EventLoop<Self>,
        token: Token, events: EventSet)
    {
        machine_loop(self, eloop, token, |m, scope| { m.ready(events, scope) })
    }

    fn notify(&mut self, eloop: &mut EventLoop<Self>, msg: Notify) {
        match msg {
            Notify::Fsm(token) => {
                machine_loop(self, eloop, token,
                    |m, scope| { m.wakeup(scope) })
            }
        }
    }

    fn timeout(&mut self, eloop: &mut EventLoop<Self>, timeo: Timeo) {
        match timeo {
            Timeo::Fsm(token) => {
                machine_loop(self, eloop, token,
                    |m, scope| { m.timeout(scope) })
            }
        }
    }
}
