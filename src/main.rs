#![feature(test)]

extern crate test;

use std::{collections::HashMap, time, sync::{Mutex, Arc}};

use rand::seq::SliceRandom;
use rayon::prelude::*;
use indicatif::ProgressIterator;

const ITERATIONS : u32 = 1_000_000_000;	

fn main() {    
    let hands: Mutex<HashMap<Combination, Vec<[Card; 7]>>> = Mutex::new(HashMap::new());

    let timer = time::Instant::now();

    let deck = Arc::new(Deck::new());

    (0..ITERATIONS).progress().par_bridge().for_each(|_| {
        let hand = Deck::new_from_cards(deck.as_ref().cards.clone()).deal();
        let result = handle_the_hand(&hand);
        if result != Combination::HighCard {
            hands.lock().unwrap().entry(result).or_insert(Vec::new()).push(hand.try_into().unwrap());
        }
    });

    println!("Finished Simulation in {:?}", timer.elapsed());

    let hands = hands.lock().unwrap();

    let total = (hands.values().map(|v| v.len()).sum::<usize>() as u32).clone();
    for i in hands.keys() {
        println!("{:?}: {}% | {}", i, (hands.get(i).unwrap().len() as u128 * 100) as f64 / ITERATIONS as f64, hands.get(i).unwrap().len());
    }
    println!("HighCards: {}% | {}", ((ITERATIONS - total) as u64 * 100 / ITERATIONS as u64) as f64, ITERATIONS - total);

}

fn handle_the_hand(hand: &Vec<Card>) -> Combination {
    match hand {
        _ if is_royal_flush(&hand) => Combination::RoyalFlush,
        _ if is_straight_flush(&hand) => Combination::StraightFlush,
        _ if is_four_of_a_kind(&hand) => Combination::FourOfAKind,
        _ if is_full_house(&hand) => Combination::FullHouse,
        _ if is_flush(&hand) => Combination::Flush,
        _ if is_straight(&hand) => Combination::Straight,
        _ if is_three_of_a_kind(&hand) => Combination::ThreeOfAKind,
        _ if is_two_pairs(&hand) => Combination::TwoPairs,
        _ if is_pair(&hand) => Combination::Pair,
        _ => Combination::HighCard
    }
}

fn is_flush(hand: &Vec<Card>) -> bool {
    hand.clone().tap(|v| v.sort_by_key(|card| card.suit)).windows(5).any(| cards| cards.iter().all(|card| card.suit == cards[0].suit))
}

fn is_straight(hand: &Vec<Card>) -> bool {
    hand.iter().map(|card| card.value).collect::<Vec<u8>>().tap(| v | v.sort()).windows(5).any(|x| x[0] + 1 == x[1] && x[1] + 1 == x[2] && x[2] + 1 == x[3] && x[3] + 1 == x[4])
}

fn is_straight_flush(hand: &Vec<Card>) -> bool {
    //get all the cards of the same suit and check if they are in a straight
    hand.clone().tap(|v| v.sort_by_key(|card| card.suit)).windows(5).any(| cards| cards.iter().all(|card| card.suit == cards[0].suit).then(|| is_straight(&cards.to_vec())).unwrap_or(false))
}

fn is_royal_flush(hand: &Vec<Card>) -> bool {
    is_flush(hand) && hand.iter().map(|card| card.value).collect::<Vec<u8>>().tap(|x| x.sort()) == vec![10, 11, 12, 13, 14]
}

fn is_four_of_a_kind(hand: &Vec<Card>) -> bool {
    hand.iter().map(|card| card.value).collect::<Vec<u8>>().tap(|v| v.sort()).windows(4).any(|x| x[0] == x[1] && x[1] == x[2] && x[2] == x[3])
}

fn is_three_of_a_kind(hand: &Vec<Card>) -> bool {
    hand.iter().map(|card| card.value).collect::<Vec<u8>>().tap(|v| v.sort()).windows(3).any(|x| x[0] == x[1] && x[1] == x[2])
}

fn is_pair(hand: &Vec<Card>) -> bool {
    hand.iter().map(|card| card.value).collect::<Vec<u8>>().tap(|v| v.sort()).windows(2).any(|x| x[0] == x[1])
}

fn is_two_pairs(hand: &Vec<Card>) -> bool {
    hand.iter().map(|card| card.value).collect::<Vec<u8>>().tap(|v| v.sort()).windows(4).filter(|x| x[0] == x[1]).count() == 2 && !is_three_of_a_kind(hand)
}

fn is_full_house(hand: &Vec<Card>) -> bool {
    hand.iter().map(|card| card.value).collect::<Vec<u8>>().tap(|v| v.sort()).windows(4).filter(|x| x[0] == x[1]).count() == 2 && is_three_of_a_kind(hand)
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Hash)]
enum Combination {
    RoyalFlush,
    StraightFlush,
    FourOfAKind,
    FullHouse,
    Flush,
    Straight,
    ThreeOfAKind,
    TwoPairs,
    Pair,
    HighCard,
}

#[derive(Debug, Clone, Copy)]
struct Card {
    value: u8,
    suit: u8,
}

impl Card {
    fn new(value: u8, suit: u8) -> Card {
        Card { value, suit }
    }
}

#[derive(Debug, Clone)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    fn new() -> Deck {
        let mut cards = Vec::new();
        for suit in 1..=4 {
            for value in 2..=14 {
                cards.push(Card::new(value, suit));
            }
        }
        cards.shuffle(&mut rand::thread_rng());
        Deck { cards: cards.clone() }
    }

    fn new_from_cards(cards: Vec<Card>) -> Deck {
        Deck { cards: cards.tap(|v| v.shuffle(&mut rand::thread_rng())) }
    }

    fn deal(&mut self) -> Vec<Card> {
        let mut hand = Vec::new();
        for i in 0..7 {
            hand.push(self.cards[i]);
        }
        hand
    }
}

trait Tap {
    fn tap(self, f: impl FnMut(&mut Self)) -> Self;
}

impl<T> Tap for T {
    fn tap(mut self, mut f: impl FnMut(&mut Self)) -> Self {
        f(&mut self);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[bench]
    fn bench_deal(b: &mut test::Bencher) {
        let mut deck = Deck::new();
        b.iter(|| deck.deal());
    }
}