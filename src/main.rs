mod bot;
mod field;

use crate::field::FieldCellState;
use field::Field;
use gloo_file::callbacks::FileReader;
use gloo_file::File;
use gloo_timers::callback::Interval;
use web_sys::{Event, HtmlInputElement};
use yew::{html, html::TargetCast, Component, Context, Html, NodeRef};

pub enum Msg {
    Loaded(String),
    File(File),
    Create,
    Step,
    Tick,
    TurnPlay,
    AddSpeed(i32),
    TurnWall(i32, i32),
    DefaultBot
}

pub struct App {
    field: Option<Field>,
    reader: Option<FileReader>,
    play_is_on: bool,
    _interval: Interval,
    file_input_ref: NodeRef,
    width_input_ref: NodeRef,
    height_input_ref: NodeRef,
    walls_input_ref: NodeRef,
    error_message: String,
    speed: u32,

}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            reader: Option::None,
            field: Option::None,
            play_is_on: false,
            _interval: Self::create_interval(ctx, 10),
            file_input_ref: NodeRef::default(),
            width_input_ref: NodeRef::default(),
            height_input_ref: NodeRef::default(),
            walls_input_ref: NodeRef::default(),
            error_message: "".to_string(),
            speed: 10,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Loaded(data) => {
                self.field.as_mut().unwrap().add_bot(data);
                self.reader = Option::None;

                let el = Self::get_html_element(&self.file_input_ref);
                el.set_value("");
                true
            }

            Msg::DefaultBot => {
                self.field.as_mut().unwrap().add_bot("loop\nloop\nstep\nendLoop\nleftOrRight\nendLoop\nleft".to_string());
                true
            }

            Msg::File(file) => {
                log::info!("msg::file");
                let task = {
                    let link = ctx.link().clone();
                    gloo_file::callbacks::read_as_text(&file, move |res| {
                        link.send_message(Msg::Loaded(res.unwrap_or_else(|e| e.to_string())))
                    })
                };
                self.reader = Some(task);
                true
            }

            Msg::Create => {
                self.field = Some(Field::new());
                let fld = self.field.as_mut().unwrap();
                if let Ok(width) = Self::get_html_element(&self.width_input_ref)
                    .value()
                    .trim()
                    .parse()
                {
                    fld.width = width;
                } else {
                    self.error_message = "Длина кривая".to_string();
                    self.field = Option::None;
                    return true;
                }
                if let Ok(height) = Self::get_html_element(&self.height_input_ref)
                    .value()
                    .trim()
                    .parse()
                {
                    fld.height = height;
                } else {
                    self.error_message = "Высота кривая".to_string();
                    self.field = Option::None;
                    return true;
                }
                if let Ok(walls_percent) = Self::get_html_element(&self.walls_input_ref)
                    .value()
                    .trim()
                    .parse()
                {
                    fld.add_random_wall(walls_percent);
                } else {
                    self.error_message = "% стен кривой".to_string();
                    self.field = Option::None;
                    return true;
                }
                self.error_message = "".to_string();
                true
            }

            Msg::Step => {
                self.field.as_mut().unwrap().step();
                true
            }

            Msg::TurnPlay => {
                self.play_is_on = !self.play_is_on;
                true
            }

            Msg::Tick => {
                if self.play_is_on {
                    self.field.as_mut().unwrap().step();
                    true
                } else {
                    false
                }
            }

            Msg::AddSpeed(value) => {
                if value.is_negative() {
                    log::info!("is_neg");
                    let v = value as u32;
                    log::info!("v{}", v);
                    if let Option::Some(res) = self.speed.checked_sub(value.abs() as u32) {
                        log::info!("is_neg{}", res);
                        self.speed = res;
                    }
                } else if let Option::Some(res) = self.speed.checked_add(value as u32) {
                    self.speed = res;
                }
                log::info!("set {}", self.speed);
                self._interval = Self::create_interval(ctx, self.speed);
                false
            }
            Msg::TurnWall(x, y) => {
                self.field.as_mut().unwrap().turn_wall(x, y);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        return html! {
        <div style="display:flex;flex-direction:column">
            {self.build_buttons_view(ctx)}
            if !self.error_message.trim().is_empty() {
              <div style="color:red">{&self.error_message}</div>
            }
            <div style="display:flex;flex-direction:column;border-top:solid 1px black;border-left:solid 1px black;width: max-content">
              {self.build_field_view(ctx)}
            </div>
        </div> };
    }
}

impl App {
    fn create_interval(ctx: &Context<Self>, millis: u32) -> Interval {
        let callback = ctx.link().callback(|_| Msg::Tick);
        Interval::new(millis, move || callback.emit(()))
    }

    fn get_html_element(node: &NodeRef) -> HtmlInputElement {
        node.cast::<HtmlInputElement>().unwrap()
    }

    fn build_field_view(&self, ctx: &Context<Self>) -> Html {
        if self.field.is_none() {
            return html! {};
        }

        let html = (0..self.field.as_ref().unwrap().height)
            .map(|idx| {
                html! {
                    <div style="display:flex">{self.build_row_view(ctx, idx) }</div>
                }
            })
            .collect::<Html>();
        html! { html }
    }

    fn build_row_view(&self, ctx: &Context<Self>, row_idx: i32) -> Html {
        (0..self.field.as_ref().unwrap().width)
            .map(|idx| html!{
               <div style="display:flex;flex-wrap:nowrap;height:40px;width:40px;border-right:1px solid black;border-bottom:1px solid black">
                {self.build_cell_view(ctx, idx, row_idx)}
                </div>
            }).collect::<Html>()
    }

    fn build_cell_view(&self, ctx: &Context<Self>, x: i32, y: i32) -> Html {
        let cell_state = self.field.as_ref().unwrap().get_cell_state(x, y);
        let style = match cell_state {
            Some(FieldCellState::Bot(color)) => format!("width:40px;height:40px;background-color:{}", color),
            Some(FieldCellState::Wall) => "width:40px;height:40px;background-color:green".to_string(),
            Option::None => "width:40px;height:40px".to_string(),
        };
        return html! {
            <div style={style} onclick={ctx.link().callback(move |_| Msg::TurnWall(x,y))}></div>
        };
    }

    fn build_buttons_view(&self, ctx: &Context<Self>) -> Html {
        let on_change_file_input = {
            ctx.link().callback(move |e: Event| {
                let input: HtmlInputElement = e.target_unchecked_into();
                if let Some(files) = input.files() {
                    let file = js_sys::try_iter(&files)
                        .unwrap()
                        .unwrap()
                        .next()
                        .unwrap()
                        .unwrap();
                    let result = File::from(web_sys::File::from(file));
                    return Msg::File(result);
                }
                panic!();
            })
        };

        html! {
            <div style="display:flex;padding-bottom:10px;margin-bottom:10px;border-bottom:1px solid black">
              <div style="display:flex;width:150px;flex-direction:column">
                 <div style="display:flex;justify-content:flex-end;margin-bottom:3px">
                    {"Длина:"}
                    <input ref={self.width_input_ref.clone()} style="width:50px;margin-left:5px" type="number" value="15" max="50" min="2" /></div>
                 <div style="display:flex;justify-content:flex-end;margin-bottom:3px">
                    {"Высота:"}
                    <input ref={self.height_input_ref.clone()} style="width:50px;margin-left:5px"  type="number" value="15" max="50" min="2"/></div>
                 <div style="display:flex;justify-content:flex-end">
                    {"Стены %:"}
                    <input ref={self.walls_input_ref.clone()} style="width:50px;margin-left:5px"  type="number" value="30" max="90" min="0"/></div>
              </div>
              <div style="display:flex; width:100px;margin-left:10px">
                 <button class="button" onclick={ctx.link().callback(|_| Msg::Create)}>{"Создать"}</button>
              </div>
             if self.field.is_some() {
                 <div style="display:flex;flex-direction:column; width:200px;margin-left:10px">
                   <div style="display:flex;flex-wrap:no-wrap;margin-bottom:3px">
                        <input type="file" multiple=false onchange={on_change_file_input} ref={self.file_input_ref.clone()} />
                   </div>
                   <div style="display:flex;align-items:center; margin-bottom:3px">
                     <div style="margin-right:5px">{"Cкорость:"}</div>
                     {self.speed}
                     <button class="button" style="margin:0 5px 0 5px" onclick={ctx.link().callback(|_| Msg::AddSpeed(1))}>{"+"}</button>
                     <button class="button" onclick={ctx.link().callback(|_| Msg::AddSpeed(-1))}>{"-"}</button>
                   </div>
                   if self.field.as_ref().unwrap().get_bots_count() > 0 {
                     <button class="button" style="width:50px" onclick={ctx.link().callback(|_| Msg::TurnPlay)}> if self.play_is_on {{"Стоп"}} else {{"Старт"}} </button>
                   }
                </div>
                <div style="display:flex;flex-direction:column">
                   <button class="button" onclick={ctx.link().callback(|_| Msg::DefaultBot)}>{"Дефолтный бот"}</button>
                </div>
             }
            <div style="display:flex;justify-content:flex-end;flex-grow:1;">
                <a href="https://github.com/zzzabl/yew-bots">{"github.com/zzzabl/yew-bots"}</a>
            </div>
            </div>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}
#[cfg(test)]
mod tests {
    use crate::Field;

    #[test]
    fn do_test() {
        let mut fld = Field::new();
        fld.width = 3;
        fld.height = 3;
        //fld.add_random_wall(2);
        fld.add_bot("loop\nloop\nstep\nendLoop\nright\nendLoop".to_string());
        loop {
            fld.step();
        }
    }
}
