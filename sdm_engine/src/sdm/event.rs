pub trait Event {
    fn name(&self) -> &str;

    fn execute(&mut self);
}

#[macro_export]
macro_rules! EventWrapper {
    ( $vis:vis struct $name:ident $({ $($varname:ident : $type:ty),* $(,)? })? ;
      $( @execute = |$exec_var:ident| $exec_code:block ; )?
    ) => {
        $vis struct $name {
            name: String,
            id: uuid::Uuid,
            exec: Option<fn(&mut Self) -> ()>,
            $($(
                $varname: $type,
            )*)?
        }

        impl Event for $name {
            fn name(&self) -> &str {
                &self.name
            }

            fn execute(&mut self) {
                if let Some(func) = self.exec {
                    func(self);
                }
            }
        }

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
