#[deny(state_machine)]

struct Phone;

impl Phone {
    fn begin_incoming_call(&mut self) {
        unimplemented!()
    }
    fn accept_call(&mut self) {
        unimplemented!()
    }
    fn end_call(&mut self) {
        unimplemented!()
    }
    fn begin_outgoing_call(&mut self) {
        unimplemented!()
    }
    fn finished_dialing(&mut self) {
        unimplemented!()
    }
    fn call_accepted(&mut self) {
        unimplemented!()
    }
}

fn main() {
    let mut x = Phone;
    x.begin_incoming_call();
    x.end_call(); //~ ERROR not applicable for states {"Ringing"}
}
