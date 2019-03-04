
pub struct Schedule {
    machines: u32,
}

pub struct OrderedJobs {
    machines: u32,

}


impl Into<Schedule> for OrderedJobs {
    fn into(self) -> Schedule {
        Schedule {
            machines: self.machines
        }
    }
}