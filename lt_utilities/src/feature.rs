pub type OscAddress = &'static str;

#[macro_export]
macro_rules! feature {
    ($name:ident) => {
        pub type $name = f32;

        paste::paste! {
            pub const [<OSC_ADDR_$name:upper>]: feature::OscAddress = concat!("/lt/", stringify!($name)); 
        }

        paste::paste! {
            pub struct [<$name Atomic>] {
                value: std::sync::atomic::AtomicU32,
            }

            impl [<$name Atomic>] {
                pub fn new(val: f32) -> Self {
                    Self {
                        value: std::sync::atomic::AtomicU32::new((val).to_bits()),
                    }
                }
    
                pub fn get(&self) -> f32 {
                    f32::from_bits(self.value.load(Ordering::SeqCst))
                }
    
                pub fn set(&self, val: f32) {
                    self.value.store(val.to_bits(), Ordering::SeqCst);
                }
            }
        }
    };
}
