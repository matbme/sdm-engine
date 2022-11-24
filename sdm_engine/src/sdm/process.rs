pub trait Process {
    fn duration(&self) -> f32;

    fn name(&self) -> &str;

    fn pid(&self) -> uuid::Uuid;

    fn is_active(&self) -> bool;

    fn start(&mut self) -> f32;

    fn end(&mut self);

    fn toggle_activate(&mut self);
}

#[macro_export]
macro_rules! ProcessWrapper {
    ( $vis:vis struct $name:ident $({ $($varvis:vis $varname:ident : $type:ty),* $(,)? })? ;
      $( @on_start = |$start_var:ident| $start_code:block ; )?
      $( @on_end = |$end_var:ident| $end_code:block ; )?
    ) => {
        $vis struct $name {
            name: String,
            pid: uuid::Uuid,
            duration: Box<dyn sdm::Distrib>,
            active: bool,
            on_start: Option<fn(&mut Self) -> ()>,
            on_end: Option<fn(&mut Self) -> ()>,
            $($(
                $varname: $type,
            )*)?
        }

        impl sdm_engine::sdm::Process for $name {
            fn duration(&self) -> f32 {
                self.duration.gen()
            }

            fn name(&self) -> &str {
                &self.name
            }

            fn pid(&self) -> uuid::Uuid {
                self.pid
            }

            fn is_active(&self) -> bool {
                self.active
            }

            fn start(&mut self) -> f32 {
                if let Some(func) = self.on_start {
                    func(self);
                }

                self.duration()
            }

            fn end(&mut self) {
                if let Some(func) = self.on_end {
                    func(self);
                }
            }

            fn toggle_activate(&mut self) {
                self.active = !self.active
            }
        }

        impl $name {
            pub fn new(name: &str, duration: impl sdm::Distrib + 'static $(,$($varname: $type),*)?) -> Self {
                let mut on_start: Option<fn(&mut Self) -> ()> = None;
                let mut on_end: Option<fn(&mut Self) -> ()> = None;

                $(on_start = Some(|$start_var| $start_code);)?
                $(on_end = Some(|$end_var| $end_code);)?

                Self {
                    name: name.to_string(),
                    pid: uuid::Uuid::new_v4(),
                    duration: Box::new(duration),
                    active: false,
                    on_start: on_start,
                    on_end: on_end,
                    $($($varname,)*)?
                }
            }
        }
    };
}
