use crate::SQL;
use crate::RATE;

pub struct sql_scanner<'a>{
    pub technique: SQL,
    pub depth: &'a RATE
}

impl sql_scanner<'_>{
    pub async fn run(self) {
        println!("Fuck you fuck You  {:?}",self.technique);
    }    
}
pub fn sql_scanner(scantype: SQL) {
    println!("Fuck you fuck You  {:?}",scantype);
}