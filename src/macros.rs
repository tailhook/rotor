#[macro_export]
macro_rules! rotor_compose {
    /* TODO(tailhook) make and check generic combinators
    (pub enum $name:ident { $($x:ident ($y:ty),)* }) => {
        pub enum $name { $($x ($y),)* }
        rotor_compose!(@machine $name C [] $($x($y),)*);
    };
    (enum $name:ident { $($x:ident ($y:ty),)* }) => {
        enum $name { $($x ($y),)* }
        rotor_compose!(@machine $name $($x($y),)*);
    };
    */
    (pub enum $name:ident <$context_type:ident> { $($x:ident ($y:ty),)* }) => {
        pub enum $name { $($x ($y),)* }
        rotor_compose!(@machine $name $context_type [] $($x($y),)*);
    };
    (enum $name:ident <$context_type:ident> { $($x:ident ($y:ty),)* }) => {
        enum $name { $($x ($y),)* }
        rotor_compose!(@machine $name $context_type [] $($x($y),)*);
    };
    (@machine $name:ident $ctx_typ:ident
        [ $(<$ctx_name:ident $(: $ctx_bound:ident)*>)* ]
        $($iname:ident ($itype:ty),)*)
    => {
        impl $( <$ctx_name:$ctx_bound> )*
            $crate::Machine<$ctx_typ> for $name {
            fn register(self, scope: &mut Scope<$ctx_typ>) -> Response<Self> {
                match self {
                    $(
                        $name::$iname(m) => {
                            m.register(scope)
                                .map($name::$iname, $name::$iname)
                        }
                    )*
                }
            }
            fn ready(self, events: EventSet, scope: &mut Scope<$ctx_typ>)
                -> Response<Self>
            {
                match self {
                    $(
                        $name::$iname(m) => {
                            m.ready(events, scope)
                                .map($name::$iname, $name::$iname)
                        }
                    )*
                }
            }
            fn spawned(self, scope: &mut Scope<$ctx_typ>) -> Response<Self>
            {
                match self {
                    $(
                        $name::$iname(m) => {
                            m.spawned(scope)
                                .map($name::$iname, $name::$iname)
                        }
                    )*
                }
            }
            fn timeout(self, scope: &mut Scope<$ctx_typ>) -> Response<Self> {
                match self {
                    $(
                        $name::$iname(m) => {
                            m.timeout(scope)
                                .map($name::$iname, $name::$iname)
                        }
                    )*
                }
            }
            fn wakeup(self, scope: &mut Scope<$ctx_typ>) -> Response<Self> {
                match self {
                    $(
                        $name::$iname(m) => {
                            m.wakeup(scope)
                                .map($name::$iname, $name::$iname)
                        }
                    )*
                }
            }
        }

    }
}
