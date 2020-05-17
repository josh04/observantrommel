use std::io::BufRead;
use rand::thread_rng;

mod log;
use log::{log, log_s};

use wasm_bindgen::prelude::*;



#[wasm_bindgen]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Card {
    Liberal,
    Fascist
}

#[wasm_bindgen]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Draw(pub Card, pub Card, pub Card);

#[wasm_bindgen]
impl Draw {

    #[wasm_bindgen(constructor)]
    pub fn new(c: Card, d: Card, e: Card) -> Draw {
        Draw(c, d, e)
    }
}

#[wasm_bindgen]
#[derive(Copy, Clone)]
pub struct Deck {
    pub draw: u32,
    pub discard: u32,

    pub liberals_played: u32,
    pub fascists_played: u32,

    pub liberals_claimed: u32,
    pub fascists_claimed: u32,
}

impl Deck {
    pub fn new() -> Deck {
        Deck{draw: 17, discard: 0, liberals_played: 0, fascists_played: 0, liberals_claimed: 0, fascists_claimed: 0}
    }

    pub fn round(&mut self, drawp: Option<Draw>, card: &Card) -> bool {
        match drawp {
            Some(draw) => {
                self.draw = self.draw - 3;
                self.discard = self.discard + 2;
        
                match draw {
                    Draw(Card::Liberal, Card::Liberal, Card::Liberal) => { 
                        self.liberals_claimed = self.liberals_claimed + 3; 
                    },
                    Draw(Card::Fascist, Card::Liberal, Card::Liberal)
                    | Draw(Card::Liberal, Card::Fascist, Card::Liberal)
                    | Draw(Card::Liberal, Card::Liberal, Card::Fascist) => {
                        self.liberals_claimed = self.liberals_claimed + 2; 
                        self.fascists_claimed = self.fascists_claimed + 1; 
                    },
                    Draw(Card::Liberal, Card::Fascist, Card::Fascist)
                    | Draw(Card::Fascist, Card::Liberal, Card::Fascist)
                    | Draw(Card::Fascist, Card::Fascist, Card::Liberal) => {
                        self.liberals_claimed = self.liberals_claimed + 1; 
                        self.fascists_claimed = self.fascists_claimed + 2; 
                    },
                    Draw(Card::Fascist, Card::Fascist, Card::Fascist) => {
                        self.fascists_claimed = self.fascists_claimed + 3; 
                    },
                }
            },
            None => {
                self.draw = self.draw - 1;
            }
        }
        

        match card {
            Card::Liberal => {
                self.liberals_played = self.liberals_played + 1;
                if drawp.is_some() && drawp.unwrap() != Draw(Card::Fascist, Card::Fascist, Card::Fascist) {
                    self.liberals_claimed = self.liberals_claimed - 1; 
                }
            },
            Card::Fascist => 
            {
                self.fascists_played = self.fascists_played + 1;
                if drawp.is_some() && drawp.unwrap() != Draw(Card::Liberal, Card::Liberal, Card::Liberal) {
                    self.fascists_claimed = self.fascists_claimed - 1; 
                }
            },
        };

        if self.draw < 3 {
            self.draw += self.discard;
            self.discard = 0;
            self.liberals_claimed = 0;
            self.fascists_claimed = 0;
            return true;
        }
        false
    }
}

#[wasm_bindgen]
#[derive(Copy, Clone)]
pub struct Round {
    pub pres: u32,
    pub draw: Draw,
    pub res: Card,
    pub prob: f32,
    pub topdeck: bool,
    pub shuffled: bool,
    pub subround_prob: f32,
}
#[wasm_bindgen]
#[derive(Clone)]
pub struct GameState {
    pub deck: Deck,
    pub current_round: u32,
    rounds: Vec<Round>,
}

fn m_pick(pick: u32, from: u32) -> f64 {
    if pick > from {
        return 0.0;
    }
    (from as u64).factorial() as f64 / ((from as u64) - (pick as u64)).factorial() as f64
}

#[wasm_bindgen]
impl GameState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> GameState {
        GameState{deck: Deck::new(), current_round: 0, rounds: Vec::new()}
    }

    pub fn past_round(&self, round: usize) -> Round {
        self.rounds[round]
    }

    pub fn subround_chance(&self) -> f32 {
        let mut sum = 0.0;
        let mut rnds = 0;

        let mut lib_plays = 0;
        let mut fasc_plays = 0;

        let mut i : u32 = 0;
        let mut first = false;
        for r in self.rounds.iter().rev() {
            if r.shuffled && !first {
                first = true;
            } else if r.shuffled {
                break;
            }
            match r.res {
                Card::Liberal => lib_plays = lib_plays + 1,
                Card::Fascist => fasc_plays = fasc_plays + 1,
            }
            i = i + 1;
        }
        let start_subround = self.rounds.len() as u32 - i;
        log_s(format!("start_subround: {}", start_subround));
        let start_lib = 6 - (self.deck.liberals_played - lib_plays);
        let start_fasc = 11 - (self.deck.fascists_played - fasc_plays);

        let mut going_lib = start_lib;
        let mut going_fasc = start_fasc;
        let mut tot_remain = start_lib + start_fasc;

        let mut p = 0;
        let mut tot_prob: f64 = 1.0;
        let mut tot_chan: f64 = 1.0;
        for j in (start_subround as usize)..self.rounds.len() {
            let draw = self.rounds[j].draw;
            let (fasc, lib) = claim_to_tuple(draw);
            let prob = m_choose(lib as u32, going_lib) * m_choose(fasc as u32, going_fasc) / m_choose(3, going_lib + going_fasc);

            tot_chan = tot_chan * m_choose(3, going_lib + going_fasc);

            going_lib = going_lib - lib as u32;
            going_fasc = going_fasc - fasc as u32;
            tot_prob = tot_prob * prob;
            log_s(format!("partial chance {}: {} ({}) tot_chan: {}", j, tot_prob, prob, tot_chan));
        }

        //let tot_perm = m_pick(start_lib + start_fasc, start_lib + start_fasc);
        //let some_perm = tot_chan / tot_perm;
        //log_s(format!("some perm: {} / {} = {}", tot_perm, tot_chan, some_perm));
        //let our_perm = some_perm / tot_prob;
        //log_s(format!("our perm : {} / {} = {}", some_perm, tot_prob, our_perm));

        let chance_total = tot_prob;//(5 as u32).factorial() as f64 * tot_prob;

        //let chance_overall = m_choose(lib_plays, start_lib) * m_choose(fasc_plays, start_fasc) / chance_total;
        log_s(format!("subround chance: {}", chance_total));
        chance_total as f32
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Stats {
    rank: Vec<(f32, &'static str)>,
    pub lib_left: u32,
    pub fasc_left: u32,
    pub topdeck_lib: f32,
    pub topdeck_fasc: f32,
}

#[wasm_bindgen]
impl Stats {
    pub fn rank_str(&self, i: usize) -> String {
        self.rank[i].1.to_string()
    }

    pub fn rank_f(&self, i: usize) -> String {
        format!("{:.1}", self.rank[i].0 * 100.0)
    }

    pub fn chance(&self, claim: Draw) -> f32 {
        let tag = claim_to_string(claim);
        for pair in self.rank.iter() {
            if pair.1 == tag.as_str() {
                return pair.0;
            }
        }
        0.0
    }

    pub fn chance_topdeck(&self, res: Card) -> f32 {
        match res {
            Card::Fascist => self.topdeck_fasc,
            Card::Liberal => self.topdeck_lib,
        }
    }
}


#[wasm_bindgen]
pub fn do_claim_round(game: &mut GameState, pres: u32, claim: Draw, res: Card) -> Stats {

    let prev_stats = print_chances(&game.deck).expect("Previous stats not found");
    let shuffled = game.deck.round(Some(claim), &res);

    log("-------------------");
    log_s(format!("Round {}: {:?} {:?}", game.current_round, claim, res));
    log_s(format!("Deck left: {} discard: {} lib pl.: {} fasc pl.: {} lib cl.: {} fasc cl.: {}", game.deck.draw, game.deck.discard, game.deck.liberals_played, game.deck.fascists_played, game.deck.liberals_claimed, game.deck.fascists_claimed));
    log("-------------------");

    let stats = print_chances(&game.deck);
    
    game.current_round = game.current_round + 1;

    game.rounds.push(Round{pres: pres, draw: claim, res: res, prob: prev_stats.chance(claim), topdeck: false, shuffled: shuffled, subround_prob: 0.0});
    
    let subprob = match shuffled {
        false => 0.0,
        true => game.subround_chance(),
    };

    game.rounds.last_mut().unwrap().subround_prob = subprob;

    stats.expect("Stats not found")
}


#[wasm_bindgen]
pub fn do_topdeck(game: &mut GameState, res: Card) -> Stats {

    let prev_stats = print_chances(&game.deck).expect("Previous stats not found");
    let shuffled = game.deck.round(None, &res);

    let stats = print_chances(&game.deck);
    
    game.current_round = game.current_round + 1;

    game.rounds.push(Round{pres: 0, draw: Draw(Card::Liberal, Card::Liberal, Card::Liberal), res: res, prob: prev_stats.chance_topdeck(res), topdeck: true, shuffled: shuffled, subround_prob: 0.0});
    
    let subprob = match shuffled {
        false => 0.0,
        true => game.subround_chance(),
    };

    game.rounds.last_mut().unwrap().subround_prob = subprob;

    stats.expect("Stats not found")
}

fn main() {
    log("Secret Hitler card counter");
}

use factorial::Factorial;

fn m_choose(choose: u32, from: u32) -> f64 {
    if choose > from {
        return 0.0;
    }
    (from as u64).factorial() as f64 / ((choose as u64).factorial() * ((from as u64) - (choose as u64)).factorial()) as f64
}

fn print_chances(deck: &Deck) -> Option<Stats> {
    let mut lib_left = std::cmp::max(6 - deck.liberals_played - deck.liberals_claimed, 0);
    let mut fasc_left = std::cmp::max(11 - deck.fascists_played - deck.fascists_claimed, 0);

    if (deck.draw < 3) {
        log("Deck too small to draw?");
        return None;
    }

    // RRR
    let choose_total = m_choose(3, deck.draw);

    let rrr = m_choose(3, fasc_left) / choose_total;
    let rbb = (m_choose(1, fasc_left) * m_choose(2, lib_left)) / choose_total;
    let rrb = (m_choose(2, fasc_left) * m_choose(1, lib_left)) / choose_total;
    let bbb = m_choose(3, lib_left) / choose_total;


    let td_total = m_choose(1, deck.draw);
    let topdeck_lib = m_choose(1, lib_left) / td_total;
    let topdeck_fasc = m_choose(1, fasc_left) / td_total;

    let mut probs = vec![(rrr as f32, "RRR"), (rbb as f32, "RBB"), (rrb as f32, "RRB"), (bbb as f32, "BBB")];

    probs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    log_s(format!(" deck: {} lib_left: {} fasc_left: {} ", deck.draw, lib_left, fasc_left));
    for (val, label) in probs.iter() {
        log_s(format!(" {}: {:.1}%", label, val * 100.0));
    }
    log_s(format!(" tot: {:.3}", rrr + rbb + rrb + bbb));

    Some(Stats{rank: probs, lib_left: lib_left, fasc_left: fasc_left, topdeck_lib: topdeck_lib as f32, topdeck_fasc: topdeck_fasc as f32})
}

fn read_claim() -> String {
    let stdin = std::io::stdin();
    let mut iterator = stdin.lock().lines();

    log("Enter the pres. claim: ");
    loop {
        let line1 = iterator.next().unwrap().unwrap();
        match line1.as_ref() {
            "RRR" | "RRB" | "RBB" | "BBB" 
            | "BBR" | "BRR" | "BRB" | "RBR" => break line1,
            _ => continue,
        }
    }
}

fn read_result() -> Card {
    let stdin = std::io::stdin();
    let mut iterator = stdin.lock().lines();

    log("Enter the result: ");

    loop {
        let line1 = iterator.next().unwrap().unwrap();
        match line1.as_ref() {
            "R" => break Card::Fascist,
            "B" => break Card::Liberal,
            _ => continue,
        }
    }
}

fn string_to_claim(line: &String) -> Option<Draw> {
    match line.as_ref() {
        "RRR" => Some(Draw(Card::Fascist, Card::Fascist, Card::Fascist)),

        "RRB" => Some(Draw(Card::Fascist, Card::Fascist, Card::Liberal)),
        "RBB" => Some(Draw(Card::Fascist, Card::Liberal, Card::Liberal)),
        "BBB" => Some(Draw(Card::Liberal, Card::Liberal, Card::Liberal)),

        "BBR" => Some(Draw(Card::Liberal, Card::Liberal, Card::Fascist)),
        "BRR" => Some(Draw(Card::Liberal, Card::Fascist, Card::Fascist)),
        "BRB" => Some(Draw(Card::Liberal, Card::Fascist, Card::Liberal)),

        "RBR" => Some(Draw(Card::Fascist, Card::Liberal, Card::Fascist)),

        _ => None,
    }
}

fn claim_to_string(claim: Draw) -> String {
    match claim {
        Draw(Card::Fascist, Card::Fascist, Card::Fascist) => "RRR",
        Draw(Card::Liberal, Card::Liberal, Card::Liberal) => "BBB",

        Draw(Card::Fascist, Card::Liberal, Card::Liberal) |
        Draw(Card::Liberal, Card::Fascist, Card::Liberal) |
        Draw(Card::Liberal, Card::Liberal, Card::Fascist) => "RBB",

        Draw(Card::Liberal, Card::Fascist, Card::Fascist) |
        Draw(Card::Fascist, Card::Liberal, Card::Fascist) |
        Draw(Card::Fascist, Card::Fascist, Card::Liberal) => "RRB",
    }.to_string()
}

fn claim_to_tuple(claim: Draw) -> (usize, usize) {
    match claim {
        Draw(Card::Fascist, Card::Fascist, Card::Fascist) => (3, 0),
        Draw(Card::Liberal, Card::Liberal, Card::Liberal) => (0, 3),

        Draw(Card::Fascist, Card::Liberal, Card::Liberal) |
        Draw(Card::Liberal, Card::Fascist, Card::Liberal) |
        Draw(Card::Liberal, Card::Liberal, Card::Fascist) => (1, 2),

        Draw(Card::Liberal, Card::Fascist, Card::Fascist) |
        Draw(Card::Fascist, Card::Liberal, Card::Fascist) |
        Draw(Card::Fascist, Card::Fascist, Card::Liberal) => (2, 1),
    }
}