use crate::card::card::Card;
use crate::hand::hand::Hand;
use crate::hand::rank::Rank;
use crate::card::value::Value;
#[derive(Debug)]
pub struct HandCalculate{
    pub hand_of_7: Hand,
    pub hand_of_5: Vec<Hand>,
}

impl HandCalculate{

    pub fn new(hand_of_7: Hand) -> Self{
        let mut hand = HandCalculate {
            hand_of_7,
            hand_of_5: Vec::<Hand>::new(),
        };

        hand.create_all_hands();

        hand
    }

    pub fn create_all_hands(&mut self){
        let mut result = Vec::<Hand>::new();
        let mut current = Vec::new();
        self.back_track(5, 0, &mut current, &mut result);
        self.hand_of_5 = result
    }

    fn back_track(&self, n: usize, start: usize, current: &mut Vec<Card>, result: &mut Vec<Hand>){
        if current.len() == n{
            result.push(Hand::new(current.clone()));
            return;
        }
        for i in start..self.hand_of_7.cards.len(){
            current.push(self.hand_of_7.cards[i].clone());
            self.back_track(n,i+1,current,result);
            current.pop();
        }
    }

    fn calculate_hands(&mut self){
        for hand in &mut self.hand_of_5{
            let flush: bool = HandCalculate::is_flush(&hand);
            let straight: Value = HandCalculate::straight(&hand);
            let four: Value = HandCalculate::four(&hand);
            let three: Value = HandCalculate::tree(&hand);
            let pair: Vec<Value> = HandCalculate::pair_number(&hand);

            if flush && straight.get_number_value() > 0{
                hand.set_rank(Rank::StraightFlush);
                hand.add_value(straight);
                continue
            }
            else if four.get_number_value() > 0{
                hand.set_rank(Rank::FourOfAKind);
                hand.add_value(four);
                let k = HandCalculate::kickers(&hand, &[four]);
                for v in k { hand.add_value(v) }
            }
            else if flush{
                hand.set_rank(Rank::Flush);
                let mut vals: Vec<Value> = hand.cards.iter().map(|c| c.value).collect();
                vals.sort_by(|a, b| b.cmp(a));
                for v in vals{ hand.add_value(v) }
            }
            else if straight.get_number_value() > 0{
                hand.set_rank(Rank::Straight);
                hand.add_value(straight);
            }

            else if three.get_number_value() > 0{
                if pair.len() >= 2{
                    hand.set_rank(Rank::FullHouse);
                    hand.add_value(three);
                    let pair_val = pair.iter().find(|p| **p != three).unwrap();
                    for v in pair{
                        if v != three{
                            hand.add_value(v)
                        }
                    }
                }
                else{
                    hand.set_rank(Rank::ThreeOfAKind);
                    hand.add_value(three);
                    let k = HandCalculate::kickers(&hand, &[three]);
                    for v in k { hand.add_value(v) }
                }
            }
            else if pair.len() > 0{
                if pair.len() == 2{
                    hand.set_rank(Rank::DoublePair);
                    hand.add_value(pair[0]);
                    hand.add_value(pair[1]);
                    let k = HandCalculate::kickers(&hand, &[pair[0], pair[1]]);
                    for v in k{ hand.add_value(v) }
                }
                else{
                    hand.set_rank(Rank::Pair);
                    hand.add_value(pair[0]);
                    let k = HandCalculate::kickers(&hand, &[pair[0]]);
                    for v in k{ hand.add_value(v) }
                }
            }
            else {
                // high value
                hand.set_rank(Rank::HighCard);
                let mut vals: Vec<Value> = hand.cards.iter().map(|c| c.value).collect();
                vals.sort_by(|a, b| b.cmp(a));
                for v in vals{ hand.add_value(v) }
            }
        }
    }

    pub fn kickers(hand: &Hand, exclude: &[Value]) -> Vec<Value>{
        let mut k: Vec<Value> = hand.cards.iter().filter(|c| !exclude.contains(&c.value)).map(|c| c.value).collect();
        k.sort_by(|a, b| b.cmp(a));
        k
    }

    pub fn best_hands(&mut self) -> Hand{
        self.calculate_hands();
        let mut best_hand: Hand = self.hand_of_5[0].clone();
        for hand in &self.hand_of_5{
            if hand.better_than(&best_hand){
                best_hand = hand.clone();
            }
        }
        best_hand
    }

    fn pair_number(hand: &Hand) -> Vec<Value>{
        let mut values = Vec::<Value>::new();
        for index in 0..hand.cards.len(){
            for index2 in index+1..hand.cards.len(){
                if hand.cards[index].value == hand.cards[index2].value && !values.contains(&hand.cards[index].value){
                    values.push(hand.cards[index].value)
                }
            }
        }
        values.sort_by(|a, b| b.get_number_value().cmp(&a.get_number_value()));
        values
    }

    fn tree(hand: &Hand) -> Value{
        for index in 0..hand.cards.len(){
            for index2 in index+1..hand.cards.len(){
                for index3 in index2+1..hand.cards.len(){
                    if hand.cards[index].value == hand.cards[index2].value && hand.cards[index2].value == hand.cards[index3].value{
                        return hand.cards[index].value
                    }
                }
            }
        }
        Value::Null
    }

    fn four(hand: &Hand) -> Value{
        for index in 0..hand.cards.len(){
            for index2 in index+1..hand.cards.len(){
                for index3 in index2+1..hand.cards.len(){
                    for index4 in index3+1..hand.cards.len(){
                        if hand.cards[index].value == hand.cards[index2].value && hand.cards[index2].value == hand.cards[index3].value && hand.cards[index3].value == hand.cards[index4].value{
                            return hand.cards[index].value
                        }
                    }
                }
            }
        }
        Value::Null
    }

    fn straight(hand: &Hand) -> Value{
        let mut list_values = Vec::<Value>::new();
        for card in &hand.cards{
            list_values.push(card.value)
        }
        list_values.sort();

        let mut is_straight = true;
        for index in 0..list_values.len()-1{
            if list_values[index].get_number_value() != list_values[index+1].get_number_value()-1{
                is_straight = false;
                break
            }
        }
        if is_straight{
            return list_values[4]
        }

        let nums: Vec<i32> = list_values.iter().map(|v| v.get_number_value()).collect();
        if nums == vec![2, 3, 4, 5, 14]{
            return Value::Five;
        }
        Value::Null
    }

    fn is_flush(hand: &Hand) -> bool{
        if hand.cards.is_empty(){
            return false
        }
        let color = hand.cards[0].color;
        hand.cards.iter().all(|c| c.color == color)
    }
}