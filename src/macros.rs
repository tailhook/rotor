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
    (pub enum $name:ident/$cname:ident <$context_type:ident>
        { $($x:ident ($y:ty),)* })
    => {
        pub enum $name { $($x ($y),)* }
        rotor_compose!(@machine $name/$cname
            $context_type [] $($x($y),)*);
    };
    (enum $name:ident/$cname:ident <$context_type:ident>
        { $($x:ident ($y:ty),)* })
    => {
        enum $name { $($x ($y),)* }
        rotor_compose!(@machine $name/$cname
            $context_type [] $($x($y),)*);
    };
    (@machine $name:ident/$cname:ident $ctx_typ:ident
        [ $(<$ctx_name:ident $(: $ctx_bound:ident)*>)* ]
        $($iname:ident ($itype:ty),)*)
    => {
        enum $cname {
            $( $iname (<$itype as $crate::Machine>::Seed), )*
        }
        impl $( <$ctx_name:$ctx_bound> )* $crate::Machine for $name {
            type Context = $ctx_typ;
            type Seed = $cname;
            fn create(seed: $cname, scope: &mut $crate::Scope<$ctx_typ>)
                -> Result<Self, Box<::std::error::Error>>
            {
                match seed {
                    $( $cname::$iname (x)
                        => $crate::Machine::create(x, scope).map($name::$iname),
                    )*
                }
            }
            fn ready(self, events: $crate::EventSet,
                scope: &mut $crate::Scope<$ctx_typ>)
                -> $crate::Response<Self, Self::Seed>
            {
                match self {
                    $(
                        $name::$iname(m) => {
                            m.ready(events, scope)
                                .map($name::$iname, $cname::$iname)
                        }
                    )*
                }
            }
            fn spawned(self, scope: &mut $crate::Scope<$ctx_typ>)
                -> $crate::Response<Self, Self::Seed>
            {
                match self {
                    $(
                        $name::$iname(m) => {
                            m.spawned(scope)
                                .map($name::$iname, $cname::$iname)
                        }
                    )*
                }
            }
            fn timeout(self, scope: &mut $crate::Scope<$ctx_typ>)
                -> $crate::Response<Self, Self::Seed>
            {
                match self {
                    $(
                        $name::$iname(m) => {
                            m.timeout(scope)
                                .map($name::$iname, $cname::$iname)
                        }
                    )*
                }
            }
            fn wakeup(self, scope: &mut $crate::Scope<$ctx_typ>)
                -> $crate::Response<Self, Self::Seed>
            {
                match self {
                    $(
                        $name::$iname(m) => {
                            m.wakeup(scope)
                                .map($name::$iname, $cname::$iname)
                        }
                    )*
                }
            }
        }

    }
}
