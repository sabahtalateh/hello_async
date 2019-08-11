//#![feature(label_break_value)]

use std::fmt::Display;

fn announce<'a, T>(value: &'a T)
where
    T: Display,
{
    println!("Behold! {}!", value);
}

fn main() {
    {
        let num = 42;
        {
            let num_ref = &num;
            announce(num_ref);
        }
    }
}
