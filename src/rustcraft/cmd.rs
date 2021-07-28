
use crate::prelude::*;
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
    Give { id: String, count: usize },
    Summon { id: String },
}

impl std::str::FromStr for Cmd {
    type Err = ();
    fn from_str(s:&str) -> Result<Self, ()> {
        let s = Scanner::new(s);
        parse(s).map_err(|_| ())
    }
}

fn parse(mut s: Scanner) -> Result<Cmd, PErr> {
    /* if s.get_char()? == '/' */ {
        let cmd = s.get_iden()?;
        match cmd.as_str() {
            "give" => {
                let id = s.get_iden()?;
                let count = s.get_integer().unwrap_or(1);
                if count < 0 {
                    return Err(PErr);
                }
                let count = count as usize;
                return Ok(Cmd::Give{ id, count })
            },
            "summon" => {
                let id = s.get_iden()?;
                return Ok(Cmd::Summon { id })
            }
            _ => return Err(PErr)
        }
    };
    Err(PErr)
}

impl Cmd {
    pub fn exec(&self, world: &mut WorldData, idata: &data::IData,) {
        match self {
            Self::Give { id, count } => {
                if let Ok(pdata) = world.entities.ecs.query_one_mut::<&mut crate::PlayerData>(world.entities.player) {
                    let mut count = *count;
                    while count > 0 {
                        let rem = count.min(64);
                        count -= rem;
                        pdata.inventory.merge(&mut ItemStack::of(idata.content.items.get(id).clone(), rem).into());
                    }
                }
            },
            Self::Summon { id } => {
                if let Some(template) = idata.content.entities.entities.get(id) {
                    let mut builder = hecs::EntityBuilder::new();
                    let pos = Position::new((50,55,50).into(), (0.9,0.9,0.9).into());
                    let aabb = pos.get_aabb();
                    builder.add(pos);
                    template.build_all_into(&mut builder);
                    let ent = world.entities.ecs.spawn(builder.build());
                    world.entities.tree.insert(ent, ent, &aabb);
                } else {
                    println!("No such entity template {}", id);
                }
            }
        }
    }
}