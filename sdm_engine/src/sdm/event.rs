pub trait Event {}

#[macro_export]
macro_rules! EventWrapper {
    ( $vis:vis struct $name:ident $({ $($varname:ident : $type:ty),* $(,)? })? ;
      $( @execute = |$exec_var:ident| $exec_code:block ; )?
    ) => {
        #[derive(Default)]
        $vis struct $name {
            name: String,
            id: uuid::Uuid,
            exec: Option<fn(&mut Self) -> ()>,
            $($(
                $varname: $type,
            )*)?
        }

        impl Event for $name {}

        impl $name {
            pub fn new(name: &str $(,$($varname: $type),*)?) -> Self {
                let mut exec: Option<fn(&mut Self) -> ()> = None;

                $(exec = Some(|$exec_var| $exec_code);)?

                Self {
                    name: name.to_string(),
                    id: uuid::Uuid::new_v4(),
                    exec: exec,
                    $($($varname,)*)?
                }
            }
        }
    };
}
