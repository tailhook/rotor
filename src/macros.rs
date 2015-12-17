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
    (pub enum $name:ident/$cname:ident/$ename:ident <$context_type:ident>
        { $($x:ident ($y:ty),)* })
    => {
        pub enum $name { $($x ($y),)* }
        rotor_compose!(@machine $name/$cname/$ename
            $context_type [] $($x($y),)*);
    };
    (enum $name:ident/$cname:ident/$ename:ident <$context_type:ident>
        { $($x:ident ($y:ty),)* })
    => {
        enum $name { $($x ($y),)* }
        rotor_compose!(@machine $name/$cname/$ename
            $context_type [] $($x($y),)*);
    };
    (@machine $name:ident/$cname:ident/$ename:ident $ctx_typ:ident
        [ $(<$ctx_name:ident $(: $ctx_bound:ident)*>)* ]
        $($iname:ident ($itype:ty),)*)
    => {
        enum $cname {
            $( $iname ($itype), )*
        }
        enum $ename {
            $( $iname ($itype), )*
        }
        impl $( <$ctx_name:$ctx_bound> )*
            $crate::Creator<$ctx_typ> for $cname {
            type Machine = $name;
            type Error = $ename;
            fn create(self, scope: &mut Scope<$ctx_typ>)
                -> Result<Self::Machine, Self::Error>
            {
                match self {
                    $( $cname::$iname (x)
                        => x.create(scope).map($name::$iname)
                                          .map_err($ename::$iname),
                    )*
                }
            }
        }
        impl $( <$ctx_name:$ctx_bound> )*
            $crate::Machine<$ctx_typ> for $name {
            type Creator = $cname;
            fn ready(self, events: EventSet, scope: &mut Scope<$ctx_typ>)
                -> $crate::Response<Self, Self::Creator>
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
            fn spawned(self, scope: &mut Scope<$ctx_typ>)
                -> $crate::Response<Self, Self::Creator>
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
            fn timeout(self, scope: &mut Scope<$ctx_typ>)
                -> $crate::Response<Self, Self::Creator>
            {
                match self {
                    $(
                        $name::$iname(m) => {
                            m.timeout(scope)
                                .map($name::$iname, $name::$iname)
                        }
                    )*
                }
            }
            fn wakeup(self, scope: &mut Scope<$ctx_typ>)
                -> $crate::Response<Self, Self::Creator>
            {
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
