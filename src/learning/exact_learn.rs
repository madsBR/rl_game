 use ndarray::{Array4};
 use crate::game::Game;


struct Sarsa{
    game : Game,
    qs : Array4<f32>,
}


impl Sarsa{

    pub fn New(game : Game) -> Self
    {
        let qs = Array4::zeros((game.game_opts.nr_tiles_v,game.game_opts.nr_tiles_h,2,2));
        Sarsa{game : game, qs: qs}
    }

}