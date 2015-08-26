///! State machine composition macros


#[macro_export]
macro_rules! rotor_compose_state_machines {
    ($name: ident <$context:ty>  { $( $subname:ident($subtype:ty), )* }) => {
        enum $name {
            $( $subname($subtype) ),*
        }

        $(
            impl $crate::context::AsyncAddMachine<$subtype> for $context {
                fn async_add_machine(&mut self, m: $subtype) -> Result<(), $subtype> {
                    match self.async_add_machine($name::$subname(m)) {
                        Err($name::$subname(m)) => Err(m),
                        Err(_) => unreachable!(),
                        Ok(()) => Ok(())
                    }
                }
            }
        )*

        impl $crate::EventMachine<Context> for $name {
            fn ready(self, events: ::mio::EventSet, context: &mut Context)
                -> Option<Self>
            {
                match self {
                    $(
                        $name::$subname(m) => m.ready(events, context)
                                               .map($name::$subname),
                    )*
                }
            }
            fn register<H: ::mio::Handler>(&mut self,
                tok: ::mio::Token, eloop: &mut ::mio::EventLoop<H>)
                -> Result<(), ::std::io::Error>
            {
                match self {
                    $(
                        &mut $name::$subname(ref mut m)
                        => m.register(tok, eloop),
                    )*
                }
            }
        }
    };
}
