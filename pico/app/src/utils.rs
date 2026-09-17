#[macro_export]
macro_rules! i2c_regs {
	($addr:literal; $($name:ident: $reg_addr:expr),+) => {
		#[allow(unused)]
		pub const ADDR: u8 = $addr;

		$(
			#[allow(unused)]
			pub const $name: u8 = $reg_addr;
		)+

		#[allow(unused)]
		pub const fn reg_name(reg_addr: u8) -> Option<&'static str> {
			match reg_addr {
				$(
					$name => Some(stringify!($name)),
				)+
				_ => None
			}
		}
	};
}
