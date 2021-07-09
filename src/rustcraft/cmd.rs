
use crate::Data;
use scanlex::*;

pub struct PErr;
impl From<ScanError> for PErr {
    fn from(_:ScanError) -> Self {Self}
}
impl From<()> for PErr {
    fn from(_:()) -> Self {Self}
}

#[derive(Debug)]
pub enum Cmd {
    Give { id: usize, count: usize }
}

impl std::str::FromStr for Cmd {
    type Err = ();
    fn from_str(s:&str) -> Result<Self, ()> {
        let s = Scanner::new(s);
        parse(s).map_err(|_| ())
    }
}

fn parse(mut s: Scanner) -> Result<Cmd, PErr> {
    if s.get_char()? == '/' {
        let cmd = s.get_iden()?;
        match cmd.as_str() {
            "give" => {
                let id = s.get_integer()?;
                if id < 0 {
                    return Err(PErr);
                }
                let id = id as usize;
                let count = s.get_integer()?;
                if count < 0 {
                    return Err(PErr);
                }
                let count = count as usize;
                return Ok(Cmd::Give{ id, count })
            },
            _ => return Err(PErr)
        }
    };
    Err(PErr)
}

impl Cmd {
    pub fn exec(&self, data: &mut Data) {
        match self {
            Self::Give{id,count} => {
                if let Ok(pdata) = data.ecs.query_one_mut::<&mut crate::PlayerData>(data.cam) {
                    let mut count = *count;
                    while count > 0 {
                        let rem = count.min(64);
                        count -= rem;
                        pdata.inventory.merge(crate::item::ItemStack::of(data.registry.get(*id), rem).into());
                    }
                }
            }
        }
    }
}