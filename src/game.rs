use std::collections::VecDeque;
use ndarray::Data;
use rand::{Rng, random};
type RewardType = i64;
#[derive(Clone, Copy,Debug)]
#[repr(u8)]
pub enum Action{
    Left,
    Up,
    Right,
    Down,
}

#[derive(Default,Clone,PartialEq,Eq)]
pub struct State {
    pub x : usize,
    pub y : usize,
    pub corner_ur: bool,
    pub corner_ll: bool,
}


pub struct DataPoint{
    pub state : State,
    pub action : Action,
    pub Reward : RewardType,
    pub new_state : State,
    pub done : bool
}

pub struct GameOptions{
    pub nr_tiles_h : usize,
    pub nr_tiles_v : usize
}


pub struct Game {
pub state : State,
pub reward_over_game : RewardType,
pub reward_total : RewardType,
pub reward_last : RewardType,
pub games_played : u64,
pub action_counter : u64,
pub recording : VecDeque<DataPoint>,
pub is_done : bool,
pub game_opts : GameOptions,
pub hole_low : (usize,usize),
pub hole_right : (usize,usize),
pub corner_ll : (usize,usize),
pub corner_ur : (usize,usize),

pub goal : (usize,usize),
}



impl Game{

    #[inline]
    pub fn is_state_at_loc(state : &State,(x,y) : (usize,usize)) -> bool {
        state.x == x && state.y == y
    }

    #[inline]
    fn raw_state_modification(& mut self,action : Action) -> bool{
        let legal_move : bool;
        match action {
            Action::Left  => {if self.state.x > 0  {self.state.x -= 1; legal_move = true;} else { legal_move = false;}},
            Action::Up    => {if self.state.y > 0  {self.state.y -= 1; legal_move = true;} else { legal_move = false;}},
            Action::Right => {if self.state.x < self.game_opts.nr_tiles_h-1 {self.state.x += 1; legal_move = true;} else { legal_move = false;}},
            Action::Down  => {if self.state.y < self.game_opts.nr_tiles_v-1 {self.state.y += 1; legal_move = true;} else { legal_move = false;}},            
        }
        if self.state.x +1  == self.game_opts.nr_tiles_h && self.state.y == 0 {self.state.corner_ur = true;}        
        if self.state.y +1 == self.game_opts.nr_tiles_v && self.state.x == 0 {self.state.corner_ll = true;}        
        return legal_move
    }

    pub fn step_and_reset(&mut self,action : Action){
        self.step(action);
        if self.is_done{
            self.reset();
        }
    }    


    pub fn step(&mut self,action : Action) -> DataPoint {  
        let old_state = self.state.clone();
        let legal_move : bool = self.raw_state_modification( action);
        let (reward,is_done) = self.calculate_reward_and_done(&old_state, action,legal_move);
        self.is_done = is_done;
        self.reward_last = reward;
        self.reward_over_game += reward;

        //Dummy        
        DataPoint{ state: old_state, action: action, Reward: reward, new_state: State::default(), done: self.is_done}
    }

    #[inline]
    pub fn calculate_reward_and_done(&self,old_state : &State,action : Action,legal_move : bool) -> (RewardType,bool){
        let mut reward = -10;
        let mut done = false;
        if !legal_move{ reward -=10;}

        if self.state.corner_ll && ! old_state.corner_ll {
            let rew : f64 = random::<f64>() * 80. * (self.game_opts.nr_tiles_v as f64);
            reward += rew as RewardType}

        if self.state.corner_ur && ! old_state.corner_ur {
        let rew : f64 = random::<f64>() * 55. * (self.game_opts.nr_tiles_h as f64);
        reward += rew as RewardType}
            
        if Game::is_state_at_loc(&self.state,self.hole_low) ||  Game::is_state_at_loc(&self.state,self.hole_right){
            reward -= (100 * self.game_opts.nr_tiles_h * self.game_opts.nr_tiles_v) as RewardType;
            done = true;
        } else if Game::is_state_at_loc(&self.state,self.goal){
            done = true;
        }
        (reward,done)
    }

    pub fn reset(&mut self){
        self.state = State::default();
        self.reward_over_game = 0;
        self.is_done = false;
    }

    pub fn New(game_opts :GameOptions) -> Game{
        let hole_low = (1usize,game_opts.nr_tiles_h-1usize);
        let hole_right = (game_opts.nr_tiles_v-1,1usize);
        let goal = (game_opts.nr_tiles_v-1usize,game_opts.nr_tiles_h-1usize);
        let corner_ll = (0usize,game_opts.nr_tiles_v-1usize);
        let corner_ur = (game_opts.nr_tiles_h-1usize,0usize);

        return Game{state : State::default(), reward_total : 0, action_counter : 0u64,recording : VecDeque::with_capacity(10),is_done : false,game_opts, reward_over_game : 0,games_played : 0,reward_last : 0, hole_low : hole_low, hole_right : hole_right,goal : goal, corner_ll : corner_ll, corner_ur : corner_ur};        
    }
}



