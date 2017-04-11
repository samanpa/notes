extern crate sio;

pub use self::sio as core;
pub mod plat;
pub mod logger;
//pub mod notes;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
