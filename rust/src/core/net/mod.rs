pub mod tcp;
pub mod socket;

//use names for futures.rs for now
enum Async<T> {
    Ready(T),
    NotReady,
}

type Poll<T, E> = Result<Async<T>, E>;

