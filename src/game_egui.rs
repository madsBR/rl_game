use std::{str::FromStr, ops::{Div, Deref}, hash::Hash, collections::HashMap, thread::Thread};
use std::path::{Path,PathBuf};
use glob::glob;
//use web_sys::console;
use egui::{FontImage, ColorImage, Ui, Key::{ArrowLeft,ArrowUp,ArrowDown,ArrowRight}};
use image::{GenericImage, GenericImageView, ImageBuffer, RgbImage,RgbaImage,Rgba,Pixel,GrayAlphaImage,imageops::*,DynamicImage,io::Reader};
use ndarray::{Array2,Array3,s,arr2,arr3, ArrayBase};
use crate::{gridworldbuilder::construct_grid_img, game::{GameOptions, self, Action, DataPoint}};
use crate::game::{Game,State};
use itertools::iproduct; 
use egui_extras::RetainedImage;
use chrono::{DateTime,Duration, TimeZone, Utc};

pub struct GridOptions{
    pub w : usize,
    pub h : usize,
    pub nr_tile_hori : usize,
    pub nr_tile_vert : usize,
    pub border_w : usize
}

pub fn copy_rgba_img_to_arr(img : &RgbaImage) -> Array3<u8>{
    Array3::from_shape_vec((img.height() as usize,img.width() as usize, 4usize),img.as_raw().clone()).expect("something went wrong generating coord array")
}


pub fn get_coords(grid_opts :&GridOptions) -> Array3<usize>{
    let tile_h_len : usize = grid_opts.h / grid_opts.nr_tile_hori;
    let tile_w_len : usize = grid_opts.w / grid_opts.nr_tile_vert;
    let coords1 = (0..grid_opts.nr_tile_hori).map(|x| x * tile_h_len);
    let coords2 = (0..grid_opts.nr_tile_vert).map(|x| x * tile_w_len);
    let coords = iproduct!(coords1,coords2);
    let mut buffer : Vec<usize> = Vec::with_capacity((grid_opts.nr_tile_hori)* (grid_opts.nr_tile_vert) * 2);
    for (x,y) in coords {buffer.push(x); buffer.push(y);}
    let coord_arr = Array3::from_shape_vec((grid_opts.nr_tile_vert,grid_opts.nr_tile_hori,2usize),buffer).expect("something went wrong generating coord array");
    return coord_arr
}

// #![feature(proc_macro)]

// #[macro_use]
// extern crate stdweb;



pub struct TemplateApp {
    pub name: String,
    pub age : i64,
    grid_world : RgbaImage,
    grid_world_rendered : RetainedImage,
    coords : Array3<usize>,
    img_token_map : HashMap<String,Array3<u8>>,
    grid_opts : GridOptions,
    game : Game,
    state_rendered : Option<State>,
    last_freeze_tp : DateTime<Utc>
    
}


impl TemplateApp{ 
    #[inline]
    pub fn key_to_action( key : egui::Key) -> Option<Action>{
        match key {
            ArrowLeft => Some(Action::Left),
            ArrowUp => Some(Action::Up),
            ArrowRight => Some(Action::Right),
            ArrowDown => Some(Action::Down),
            _ => None
        }
    }

    fn action_key_listeners(&mut self,ctx : &egui::Context,ui :&mut Ui) -> Option<DataPoint>{
        const ACTION_KEYS : [egui::Key;4] = [ArrowUp,ArrowRight,ArrowLeft,ArrowDown];
        let mut dt : Option<DataPoint> = None;
        if self.game.is_done{
            for key in ACTION_KEYS{
                if ctx.input().key_pressed(key){
                    self.game.reset();
                }
            }
        dt = None

        } else {
            for key in ACTION_KEYS{
                if ctx.input().key_pressed(key){
                    println!("registered key {}",key.symbol_or_name());
                    if let Some(action) = TemplateApp::key_to_action(key) {
                        println!("recieved action{:?}",action);
                        dt = Some(self.game.step(action));
                    }
                }
            }
            if self.game.is_done{
                self.when_donified()
            }
        }
        dt
    }

    #[inline]
    pub fn token_from_state(&self,old_state : &mut Option<State>, state : &State) -> String{
        if Game::is_state_at_loc(state, self.game.hole_low){
            return String::from("angry");
        }
        if Game::is_state_at_loc(state, self.game.hole_right){
            return String::from("angry");
        }
        if Game::is_state_at_loc(state, self.game.goal){
            return String::from("goal");
        }
        if Game::is_state_at_loc(state, self.game.corner_ll) && !(old_state.as_ref()).unwrap().corner_ll{
            return String::from("happy");
        }
        if Game::is_state_at_loc(state, self.game.corner_ur ) && !(old_state.as_ref()).unwrap().corner_ur{
            return String::from("happy");
        }


        return String::from("duncare");

    }
    
    pub fn when_donified(&mut self){
        
        self.last_freeze_tp = Utc::now();
    }
    pub fn generate_grid_from_state(&self,state : &State, odt :  Option<DataPoint>) -> Array3<u8>{
        let mut mb_old_state_ref = match odt {
            Some(dt) => Some(dt.state),
            None => None,        
        };
        let token = self.token_from_state(&mut mb_old_state_ref,&self.game.state);
        let token_img = &self.img_token_map[&token];
        println!("token ig shape is {:?}",token_img.shape());
        let mut coords =  self.coords.slice(s![state.y,state.x,..]).to_owned();
        coords = coords + self.grid_opts.border_w;
        let mut arr =  copy_rgba_img_to_arr(&self.grid_world);
        let shape = token_img.shape();
        println!("grid_world_arr_shape  is {:?}",arr.shape());
        arr.slice_mut(s![coords[0]..coords[0] + shape[0],coords[1] .. coords[1]+ shape[1],..]).assign(&token_img);
        return arr
    }   

    pub fn initialize_img_token_map(&mut self){        

        {
            let happy_buf = include_bytes!("imgs/happy.png");
            self.add_img_token_from_name("happy",happy_buf);
        }
        {
            let duncare_buf = include_bytes!("imgs/duncare.png");
            self.add_img_token_from_name("duncare",duncare_buf);
        }
        {
            let goal_buf = include_bytes!("imgs/goal.png");
            self.add_img_token_from_name("goal",goal_buf);
        }
        {
            let angry_buf = include_bytes!("imgs/angry.png");
            self.add_img_token_from_name("angry",angry_buf);
        }

    }

    pub fn add_img_token_from_name(&mut self,name : &str,img_buf : &[u8]){
        let pattern = format!("imgs/{}.png" , name);
        //println!("glob pattern for tokens {}",pattern);

        // let mut path : PathBuf= PathBuf::new();
        // for entry in glob(&pattern).expect("Failed to read glob pattern") {
        //     match entry {
        //         Ok(found_path) => {path.extend(&found_path); break;},
        //         Err(e) => {console::log_1(&e.to_string().into())},
        //     }        
        // }
        let path = Path::new(&pattern);
//        let stuff = format!("path is {}",&path.to_str().unwrap());
//        println!("{}",stuff);

//        console::log_1(stuff.into());
        // let url = match name {
        //     "happy" => "https://powersportsdealersupply.com/wp-content/uploads/catalog/product/ds134c2.jpg",
        //     "angry" => "https://www.kindpng.com/picc/m/2-28911_memes-colorful-rage-face-sticker-rage-png-memes.png",
        //     "goal" => "https://loquiz.com/wpmainpage/wp-content/uploads/2020/02/Untitled-3-768x589.png",
        //     _ => "https://pics.me.me/thumb_poker-face-meme-transparent-png-clipart-free-download-51585365.png",


        // };
//        let img_bytes = reqwest::blocking::get(url).expect("need url")
//        .bytes().expect("should have returned bytes");
        let x = path.to_str().unwrap();        
        
        let img = image::load_from_memory(img_buf).expect("problem with decoding").into_rgba8();

//        let img = image::open(&path).expect(&format!("could not load {} when searching for {}. DOes it exist?",path.to_str().unwrap(),name)).into_rgba8();
            
        self.add_img_token_from_img(name,img);        
    }

    pub fn add_img_token_from_img<C>(&mut self,key : &str,img : ImageBuffer<Rgba<u8>,C>)
    where C : Deref<Target=[u8]>
    {
        let img_resize = resize(&img, (self.grid_opts.w.div(self.grid_opts.nr_tile_hori) - 2* self.grid_opts.border_w) as u32, ( self.grid_opts.h.div(self.grid_opts.nr_tile_vert) - 2* self.grid_opts.border_w) as u32, FilterType::Gaussian);    
    //     let img_ret : RetainedImage = RetainedImage::from_color_image("grid",
    //         ColorImage::from_rgba_unmultiplied([img_resize.dimensions().0 as usize,img_resize.dimensions().1 as usize], &img_resize)    
    // );

        let img_arr = copy_rgba_img_to_arr(&img_resize);
        self.img_token_map.insert(String::from(key),img_arr);
        
    }

    pub fn New() -> Self { 
        let (w,h,nr_ch,nr_cv) =(500usize,300usize,7usize,7usize);
        let grid_opts : GridOptions = GridOptions{w : w, h : h, nr_tile_hori : nr_ch, nr_tile_vert : nr_cv,border_w:1};
        let game_opts : GameOptions = GameOptions { nr_tiles_h: grid_opts.nr_tile_hori, nr_tiles_v: grid_opts.nr_tile_vert};
        let name = String::from("hello");
        let age : i64 = 50;
        let img = construct_grid_img(grid_opts.w,grid_opts.h,grid_opts.nr_tile_hori,grid_opts.nr_tile_vert,1);
        let dyna_rgba_img = DynamicImage::ImageLumaA8(img).into_rgba8();
        let coords = get_coords(&grid_opts);
        let img_ret : RetainedImage = RetainedImage::from_color_image("grid",
            ColorImage::from_rgba_unmultiplied([dyna_rgba_img.dimensions().0 as usize,dyna_rgba_img.dimensions().1 as usize], &dyna_rgba_img)    
        );
        let game = Game::New(game_opts);
        let mut app = TemplateApp{name : name, age : age, grid_world : dyna_rgba_img , grid_world_rendered : img_ret,coords,grid_opts : grid_opts,img_token_map : HashMap::new(), game : game , state_rendered : None, last_freeze_tp : DateTime::<Utc>::MIN_UTC};
        app.initialize_img_token_map();                        
        return app;
    }
    
    
    pub fn allow_keys(&self) -> bool{
        let time = Utc::now();
        time.signed_duration_since(self.last_freeze_tp) > Duration::seconds(2)
    }

    pub fn update_render_frame(&self,ctx : &egui::Context,ui :&mut Ui, dt : Option<DataPoint>) -> (State,RetainedImage){
        println!("rendering");
        let arr = self.generate_grid_from_state(&self.game.state, dt);        
        let img_ret : RetainedImage = RetainedImage::from_color_image("grid",
            ColorImage::from_rgba_unmultiplied([arr.shape()[1],arr.shape()[0]], &arr.into_raw_vec()));    
        return (self.game.state.clone(),img_ret)
    }

    pub fn reset_game(&mut self,ctx : &egui::Context,ui :&mut Ui){
        self.game.reset();

    }

    pub fn update_rendering(&mut self,ctx : &egui::Context,ui :&mut Ui, dt : Option<DataPoint>){
        let mut render : bool = false;
        if self.state_rendered.is_none(){
            println!("its none");
            render = true;
        } else{
            if *self.state_rendered.as_ref().unwrap() != self.game.state {
                render = true;
            }
        }
        if render {
            let (new_state,img) = self.update_render_frame(ctx, ui, dt);
            self.state_rendered = Some(new_state);
            self.grid_world_rendered = img;
        }
        
        ui.add(egui::Image::new(self.grid_world_rendered.texture_id(ctx),self.grid_world_rendered.size_vec2()));
    }
}





impl eframe::App for TemplateApp {    
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
 
           ui.heading("A GAME");
           ui.label(egui::RichText::from(format!("Goal is to get the most points")).size(16f32));
           ui.label(egui::RichText::from(format!("Play by using the 4 arrowkeys on the keyboard")).size(16f32));
           ui.horizontal(|ui| {
//                ui.label("Your name: ");
//                ui.text_edit_singleline(&mut self.name);
            });
//            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
//            if ui.button("Click each year").clicked() {
//                self.age += 1;
//            }
            let dt : Option<DataPoint>;
            if self.allow_keys() {
                dt = self.action_key_listeners(ctx, ui);
            } else {
                dt = None
            }
            self.update_rendering(ctx,ui,dt);
            ui.label(egui::RichText::from(format!("Points awarded for your last this action: {}",self.game.reward_last)).size(24f32));
            ui.label(egui::RichText::from(format!("Total Points this game: {}",self.game.reward_over_game)).size(24f32));
            if self.game.is_done{
                ui.label(egui::RichText::from(format!("Game ended! You got : {} points! press any arrowkey to restart",self.game.reward_over_game)).size(42f32));
            }
        });
    }
}
