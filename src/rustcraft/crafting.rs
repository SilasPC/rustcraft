
use crate::item::ItemLike;
use std::collections::HashMap;
use crate::item::Item;
use crate::Registry;
use std::sync::Arc;
use crate::item::ItemStack;

#[derive(Debug)]
pub enum Node {
    Branch(HashMap<ItemLike, Node>, Option<Box<Node>>),
    Leaf(ItemStack)
}

impl Node {
    pub fn search(&self, input: &[Option<ItemStack>]) -> Option<&ItemStack> {
        //println!("search {:?}\n WITH {:?}",self, input);
        match self {
            Self::Branch(map, alt) => {
                if let Some(inp) = &input[0] {
                    if let Some(node) = map.get(&inp.item) {
                        node.search(&input[1..])
                    } else {
                        None
                    }
                } else if let Some(alt) = alt {
                    alt.search(&input[1..])
                } else {
                    None
                }
            },
            Self::Leaf(out) if input.len() == 0 => Some(&out),
            _ => None
        }
    }
    pub fn register(&mut self, input: &[Option<ItemLike>], output: ItemStack) {
        match self {
            Self::Branch(ref mut map, ref mut alt) => {
                if let Some(inp) = input.get(0) {
                    if let Some(stack) = inp {
                        if let Some(node) = map.get_mut(&stack) {
                            node.register(&input[1..], output);
                        } else {
                            let mut nnode = Node::Branch(HashMap::new(), None);
                            nnode.register(&input[1..], output);
                            map.insert(stack.clone(), nnode);
                        }
                    } else if let Some(alt) = alt {
                        alt.register(&input[1..], output);
                    } else {
                        let mut nnode = Node::Branch(HashMap::new(), None);
                        nnode.register(&input[1..], output);
                        *alt = Some(Box::new(nnode));
                    };
                } else {
                    if map.is_empty() && alt.is_none() {
                        *self = Self::Leaf(output)
                    } else {
                        panic!("Something override")
                    }
                }
            },
            Self::Leaf(_) => panic!("something")
        };
    }
}

#[derive(Debug)]
pub struct CraftingRegistry {
    shaped: Vec<Node>,
    unshaped: Vec<Node>,
}

impl CraftingRegistry {

    pub fn new() -> Self {
        Self {
            shaped: vec![Node::Branch(HashMap::new(), None)],
            unshaped: vec![Node::Branch(HashMap::new(), None)],
        }
    }

    pub fn register(&mut self, shaped: bool, input: &[Option<ItemLike>], output: ItemStack) {
        if shaped {
            self.shaped[0].register(input, output);
        } else {
            todo!("unshaped not supported until ItemLike is PartialOrd");
        }
    }

    pub fn search(&self, mut input: &[Option<ItemStack>]) -> Option<&ItemStack> {
        self.shaped[0].search(input)
    }

    pub fn get_output(&self, input: &[Option<ItemStack>], reg: &Registry) -> Option<ItemStack> {
        if input.iter().skip(1).any(Option::is_some) || input[0].is_none() {
            return None
        };
        let stack = input[0].as_ref().unwrap();
        match stack.item.id() {
            2 => Some(ItemStack::of(reg.get(7), 1)),
            _ => None
        } 
    }
}
