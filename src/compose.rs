///! State machine composition macros


#[macro_export]
macro_rules! rotor_compose_state_machines {
    (STRUCT_REPEAT $name:ident $context:ty [ $($name1:ident/$type1:ty)* ]
                               [ $($name2:ident/$type2:ty)* ]) => {
        mod scope {
            $(
                rotor_compose_state_machines!(SCOPE_STRUCT $name $name1);
            )*
        }
        rotor_compose_state_machines!(SCOPE_WRAPPER
            $name $context [ $({$name1 $type1})* ] [ $($name2/$type2)* ]);
    };
    (SCOPE_STRUCT $name:ident $cursub:ident) => {
        pub struct $cursub<'a, S: 'a>(pub &'a mut S);
    };
    (SCOPE_WRAPPER $name:ident $context:ty [ { $cursub:ident $curtyp:ty } ]
                               [ $($subname:ident / $subtype:ty)* ]) => {
        rotor_compose_state_machines!(SCOPE_IMPL
            $name $context { $cursub $curtyp } $($subname/$subtype)*);
    };
    (SCOPE_WRAPPER $name:ident $context:ty
        [ { $firstsub:ident $firsttyp:ty } $( { $tail:ident $tailtyp:ty } )* ]
        [ $($name2:ident/$type2:ty)* ]) => {
        rotor_compose_state_machines!(SCOPE_IMPL
            $name $context { $firstsub $firsttyp } $($name2/$type2)*);
        rotor_compose_state_machines!(SCOPE_WRAPPER
            $name $context [ $({ $tail $tailtyp })* ] [ $($name2/$type2)* ]);
    };
    (SCOPE_IMPL $name:ident $context:ty { $cursub:ident $curtyp:ty }
        $($subname:ident/$subtype:ty)*) => {
        impl<'a, S> $crate::Scope<$curtyp,
                    <$curtyp as $crate::EventMachine<$context>>::Timeout>
            for scope::$cursub<'a, S>
            where S: $crate::Scope<
                $name<$($subtype),*>,
                $name<$(<$subtype as $crate::EventMachine<$context>>::Timeout),*>> + 'a,
        {
            fn async_add_machine(&mut self, m: $curtyp) -> Result<(), $curtyp> {
                self.0.async_add_machine($name::$cursub(m))
                .map_err(|x| if let $name::$cursub(c) = x {
                    c
                } else {
                    unreachable!();
                })
            }
            fn add_timeout_ms(&mut self, delay: u64,
                t: <$curtyp as $crate::EventMachine<$context>>::Timeout)
                -> Result<::mio::Timeout, ::mio::TimerError>
            {
                self.0.add_timeout_ms(delay, $name::$cursub(t))
            }
            fn clear_timeout(&mut self, timeout: ::mio::Timeout) -> bool {
                self.0.clear_timeout(timeout)
            }
            fn register<E: ?Sized>(&mut self, io: &E,
                interest: ::mio::EventSet, opt: ::mio::PollOpt)
                -> Result<(), ::std::io::Error>
                where E: ::mio::Evented
            {
                self.0.register(io, interest, opt)
            }
        }
    };
    ($name: ident <$context:ty>  { $( $subname:ident($subtype:ty), )* }) => {
        pub enum $name<$($subname),*> {
            $( $subname($subname) ),*
        }

        rotor_compose_state_machines!(STRUCT_REPEAT $name $context
            [ $($subname / $subtype)* ] [ $($subname / $subtype)* ]);

        impl $crate::EventMachine<$context> for $name<$($subtype),*> {
            type Timeout = $name<$(
                <$subtype as rotor::EventMachine<$context>>::Timeout
            ),*>;
            fn ready<'x, S>(self, events: ::mio::EventSet,
                context: &mut $context, scope: &mut S)
                -> Option<Self>
                where S: 'x, S: $crate::Scope<Self, Self::Timeout>
            {
                match self {
                    $(
                        $name::$subname(m)
                        => m.ready(events, context, &mut scope::$subname(scope))
                                               .map($name::$subname),
                    )*
                }
            }
            fn register<'x, S>(&mut self, scope: &mut S)
                -> Result<(), ::std::io::Error>
                where S: 'x, S: $crate::Scope<Self, Self::Timeout>
            {
                match self {
                    $(
                        &mut $name::$subname(ref mut m)
                        => m.register(&mut scope::$subname(scope)),
                    )*
                }
            }
        }
    };
}
