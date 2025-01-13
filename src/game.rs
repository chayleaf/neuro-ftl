use std::{
    borrow::Cow,
    collections::{BTreeMap, BinaryHeap, HashMap, HashSet},
    ffi::c_int,
    ops::DerefMut,
    ptr,
    sync::OnceLock,
    time::{Duration, Instant},
};

use actions::{FtlActions, InventorySlotType, TargetShip};
use futures_util::{SinkExt, StreamExt};
use indexmap::IndexMap;
use neuro_sama::game::{Action, ApiMut};
use rand::Rng;
use serde::Serialize;
use tokio::sync::mpsc;

use crate::bindings::{self, power_manager, xc, xm, CApp, Door, System};

pub mod actions;
mod context;
pub mod strings;

fn meta<T: Action>() -> neuro_sama::schema::Action {
    neuro_sama::schema::Action {
        schema: schemars::schema_for!(T),
        name: T::name().into(),
        description: T::description().into(),
    }
}

struct State {
    tx: mpsc::Sender<tungstenite::Message>,
    rx: mpsc::Receiver<Option<tungstenite::Message>>,
    app: *mut CApp,
    actions: ActionDb,
    //buffer: context::Context,
}

unsafe impl Sync for State {}
unsafe impl Send for State {}

fn resource_event_str(
    res: &bindings::ResourceEvent,
    ship_manager: &bindings::ShipManager,
) -> String {
    let mut ret = Vec::new();
    if res.fuel > 0 {
        ret.push(format!("will get {} fuel", res.fuel));
    }
    if res.missiles > 0 {
        ret.push(format!("will get {} missiles", res.missiles));
    }
    if res.drones > 0 {
        ret.push(format!("will get {} drone parts", res.drones));
    }
    if res.scrap > 0 {
        ret.push(format!("will get {} scrap", res.scrap));
    }
    if res.weapon().is_some_and(|x| x.type_ != -1) {
        ret.push(format!(
            "will get a weapon ({})",
            res.weapon().unwrap().desc.title.to_str()
        ));
    }
    if res.drone().is_some_and(|x| x.type_ != -1) {
        ret.push(format!(
            "will get a drone ({})",
            res.drone().unwrap().desc.title.to_str()
        ));
    }
    if res.augment().is_some_and(|x| x.type_ != -1) {
        ret.push(format!(
            "will get an augment ({})",
            res.augment().unwrap().desc.title.to_str()
        ));
    }
    match res.crew {
        1 => ret.push(format!(
            "will gain a crew member ({})",
            res.crew_blue.crew_name.to_str()
        )),
        2.. => ret.push(format!(
            "will gain {} crew members ({})",
            res.crew,
            res.crew_blue.crew_name.to_str()
        )),
        ..=-1 if res.traitor => ret.push(format!(
            "{} will betray you",
            res.crew_blue.crew_name.to_str()
        )),
        ..=-1 => ret.push(format!("{} will die", res.crew_blue.crew_name.to_str())),
        0 => {}
    }
    if res.intruders {
        ret.push("intruders will board your ship".to_owned());
    }
    match res.fleet_delay {
        1 => ret.push("rebel fleet pursuit speed will be doubled for 1 jump".to_owned()),
        2.. => ret.push(format!(
            "rebel fleet pursuit speed will be doubled for {} jumps",
            res.fleet_delay
        )),
        -1 => ret.push("rebel fleet will be delayed by 1 jump".to_owned()),
        ..=-1 => ret.push(format!(
            "rebel fleet will be delayed by {} jumps",
            -res.fleet_delay
        )),
        0 => {}
    }
    if res.hull_damage > 0 {
        ret.push(format!("ship hull will take {} damage", res.hull_damage));
    }
    if !res.remove_item.to_str().is_empty() {
        let name = res.remove_item.to_str();
        ret.push(format!(
            "item will be removed ({})",
            super::library()
                .blueprint_name(&name)
                .unwrap_or_else(|| name.as_ref()),
        ));
    }
    if res.hull_damage < 0 {
        ret.push(format!(
            "ship hull will be repaired by {}",
            -res.hull_damage
        ));
    }
    if let Some(upgrade_id) = System::from_id(res.upgrade_id) {
        if ship_manager.system(upgrade_id).is_some() {
            let bp = upgrade_id.blueprint().unwrap();
            ret.push(format!("{} will be upgraded", bp.title.to_str()));
        }
    }
    if let Some(system_id) = System::from_id(res.system_id) {
        let bp = system_id.blueprint().unwrap();
        ret.push(format!("{} will be installed", bp.title.to_str()));
    }
    let ret = ret.join(", ");
    if !ret.is_empty() {
        "\nEvent effect: ".to_owned() + &ret
    } else {
        ret
    }
}

#[derive(Default)]
struct ShipGraph {
    pub rooms: HashMap<c_int, Vec<(c_int, c_int)>>,
}

impl ShipGraph {
    pub fn add_door(&mut self, id: c_int, a: c_int, b: c_int) {
        self.rooms.entry(a).or_default().push((id, b));
        self.rooms.entry(b).or_default().push((id, a));
    }
    pub fn shortest_path(
        &self,
        a: c_int,
        b: c_int,
    ) -> Result<Vec<c_int>, Option<Cow<'static, str>>> {
        let mut q = BinaryHeap::new();
        let mut vis = HashSet::new();
        q.push((usize::MAX, vec![], a));
        while let Some((level, path, room)) = q.pop() {
            if room == b {
                return Ok(path);
            }
            if !vis.insert(room) {
                continue;
            }
            let Some(room) = self.rooms.get(&room) else {
                return Err(Some(format!("room {room} doesn't exist").into()));
            };
            for (door, room) in room.iter().copied() {
                if vis.contains(&room) {
                    continue;
                }
                let mut path = path.clone();
                path.push(door);
                q.push((level - 1, path, room));
            }
        }
        Err(Some(
            format!("there's no path between rooms {a} and {b}").into(),
        ))
    }
}

impl State {
    fn app_mut(&self) -> Option<&mut CApp> {
        unsafe { xm(self.app) }
    }
}

struct IdMap<'a>(HashMap<Cow<'a, str>, usize>);

impl<'a> IdMap<'a> {
    pub fn with<T>(x: impl FnOnce(&mut Self) -> T) -> T {
        let mut this = Self::new();
        x(&mut this)
    }
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn map(&mut self, x: Cow<'a, str>) -> Cow<'a, str> {
        let v = self.0.entry(x.clone()).or_default();
        *v += 1;
        if *v < 2 {
            x
        } else {
            format!("{x} ({})", *v).into()
        }
    }
}

impl neuro_sama::game::GameMut for State {
    const NAME: &'static str = "FTL: Faster Than Light";
    type Actions<'a> = FtlActions;
    fn send_command(&mut self, message: tungstenite::Message) {
        self.tx.try_send(message).ok();
    }
    fn handle_action<'a>(
        &mut self,
        action: Self::Actions<'a>,
    ) -> Result<
        Option<impl 'static + Into<Cow<'static, str>>>,
        Option<impl 'static + Into<Cow<'static, str>>>,
    > {
        let Some(app) = self.app_mut() else {
            return Err(Cow::from("CApp is null, game is broken").into());
        };
        log::debug!("handling action: {action:?}");
        let ret: Result<Option<Cow<'static, str>>, Option<Cow<'static, str>>> = match action {
            // only main menu
            FtlActions::SkipCredits(event) => {
                if self.actions.valid(&event) {
                    if app.menu.b_open {
                        if app.menu.b_credit_screen {
                            app.menu.b_credit_screen = false;
                            Ok(Cow::from("skipped credits").into())
                        } else {
                            Err(Cow::from("credits aren't playing").into())
                        }
                    } else if app.gui().unwrap().game_over_screen.b_showing_credits {
                        app.gui_mut().unwrap().game_over_screen.b_showing_credits = false;
                        Ok(Cow::from("skipped credits").into())
                    } else {
                        Err(Cow::from("credits aren't playing").into())
                    }
                } else {
                    Err(Cow::from("credits aren't playing").into())
                }
            }
            FtlActions::NewGame(event) => {
                if self.actions.valid(&event)
                    && app.menu.b_open
                    && app.menu.start_button.base.b_active
                {
                    for btn in app.menu.buttons.iter() {
                        let btn = unsafe { xm(*btn).unwrap() };
                        btn.base.b_hover = false;
                    }
                    app.menu.start_button.base.b_hover = true;
                    unsafe {
                        app.base
                            .vtable()
                            .on_l_button_down(ptr::addr_of_mut!(app.base), 0, 0)
                    };
                    Ok(Cow::from("starting a new game").into())
                } else {
                    Err(Cow::from("can't start a new game right now").into())
                }
            }
            FtlActions::Continue(event) => {
                if self.actions.valid(&event)
                    && app.menu.b_open
                    && app.menu.continue_button.base.b_active
                {
                    for btn in app.menu.buttons.iter() {
                        let btn = unsafe { xm(*btn).unwrap() };
                        btn.base.b_hover = false;
                    }
                    app.menu.start_button.base.b_hover = false;
                    app.menu.continue_button.base.b_hover = true;
                    unsafe {
                        app.base
                            .vtable()
                            .on_l_button_down(ptr::addr_of_mut!(app.base), 0, 0)
                    };
                    Ok(Cow::from("continuing from existing save").into())
                } else {
                    Err(Cow::from("can't continue from an existing save").into())
                }
            }
            FtlActions::Confirm(_) | FtlActions::Deny(_) => {
                let (valid, confirm) = match action {
                    FtlActions::Confirm(event) => (self.actions.valid(&event), true),
                    FtlActions::Deny(event) => (self.actions.valid(&event), false),
                    _ => unreachable!(),
                };
                if !valid {
                    if confirm {
                        Err(Cow::from("nothing to confirm").into())
                    } else {
                        Err(Cow::from("nothing to deny").into())
                    }
                } else if app.menu.b_open && app.menu.confirm_new_game.base.b_open {
                    let window = &mut app.menu.confirm_new_game;
                    window.base.b_open = false;
                    window.result = confirm;
                    unsafe {
                        app.base
                            .vtable()
                            .on_l_button_down(ptr::addr_of_mut!(app.base), 0, 0)
                    };
                    if confirm {
                        Ok(Cow::from("starting a new game").into())
                    } else {
                        Ok(Cow::from("not starting a new game").into())
                    }
                } else if !app.menu.b_open && app.gui().unwrap().leave_crew_dialog.base.b_open {
                    let window = &mut app.gui_mut().unwrap().leave_crew_dialog;
                    window.base.b_open = false;
                    window.result = confirm;
                    unsafe {
                        app.base
                            .vtable()
                            .on_l_button_down(ptr::addr_of_mut!(app.base), 0, 0)
                    };
                    if confirm {
                        Ok(Cow::from("leaving crew members behind").into())
                    } else {
                        Ok(Cow::from("canceling the jump").into())
                    }
                } else if app.gui().unwrap().crew_screen.delete_dialog.base.b_open {
                    let window = &mut app.gui_mut().unwrap().crew_screen.delete_dialog;
                    window.base.b_open = false;
                    window.result = confirm;
                    unsafe {
                        app.base
                            .vtable()
                            .on_l_button_down(ptr::addr_of_mut!(app.base), 0, 0)
                    };
                    if confirm {
                        Ok(Cow::from("fired the crew member o7").into())
                    } else {
                        Ok(Cow::from("keeping the crew member").into())
                    }
                } else {
                    Err(Cow::from("nothing to confirm").into())
                }
            }
            FtlActions::RenameShip(event) => {
                if self.actions.valid(&event) {
                    app.menu.ship_builder.name_input.b_active = true;
                    let old = app
                        .menu
                        .ship_builder
                        .name_input
                        .text
                        .iter()
                        .filter_map(|x| char::from_u32(*x as u32))
                        .collect::<String>();
                    unsafe {
                        app.base
                            .vtable()
                            .on_text_event(ptr::addr_of_mut!(app.base), bindings::TextEvent::Clear);
                    }
                    for char in event.name.chars() {
                        unsafe {
                            app.base
                                .vtable()
                                .on_text_input(ptr::addr_of_mut!(app.base), char as i32);
                        }
                    }
                    app.menu.ship_builder.name_input.b_active = false;
                    Ok(Cow::from(format!(
                        "renamed the ship, old ship name is {old:?}, new ship name is {:?}",
                        event.name
                    ))
                    .into())
                } else {
                    Err(Cow::from("can't rename the ship at this time").into())
                }
            }
            FtlActions::RenameCrew(event) => {
                if self.actions.valid(&event) {
                    if app.menu.ship_builder.b_open {
                        if let Some(member) = IdMap::with(|map| {
                            app.menu.ship_builder.v_crew_boxes.iter().find(|x| {
                                unsafe { xc(**x).unwrap() }
                                    .base
                                    .base
                                    .item
                                    .crew()
                                    .is_some_and(|x| {
                                        map.map(x.blueprint.crew_name_long.to_str())
                                            == event.old_name.as_str()
                                    })
                            })
                        }) {
                            let member = unsafe { xm(*member).unwrap() };
                            member.base.b_quick_renaming = true;
                            member.base.name_input.b_active = true;
                            let old = member
                                .base
                                .name_input
                                .text
                                .iter()
                                .filter_map(|x| char::from_u32(*x as u32))
                                .collect::<String>();

                            unsafe {
                                app.base.vtable().on_text_event(
                                    ptr::addr_of_mut!(app.base),
                                    bindings::TextEvent::Clear,
                                )
                            };
                            for char in event.name.chars() {
                                unsafe {
                                    app.base
                                        .vtable()
                                        .on_text_input(ptr::addr_of_mut!(app.base), char as i32)
                                };
                            }
                            member.base.name_input.b_active = false;

                            Ok(Cow::from(format!(
                                "renamed the crew member, old name is {old:?}, new name is {:?}",
                                event.name
                            ))
                            .into())
                        } else {
                            let names =
                                IdMap::with(|map| {
                                    app.menu
                                        .ship_builder
                                        .v_crew_boxes
                                        .iter()
                                        .filter_map(|x| {
                                            unsafe { xc(*x).unwrap() }.base.base.item.crew().map(
                                                |x| map.map(x.blueprint.crew_name_long.to_str()),
                                            )
                                        })
                                        .collect::<Vec<_>>()
                                });
                            Err(Cow::from(format!(
                                "this crew member doesn't exist, current crew members: {}",
                                serde_json::to_string(&names).unwrap()
                            ))
                            .into())
                        }
                    } else {
                        let c = IdMap::with(|map| {
                            app.gui()
                                .unwrap()
                                .ship_manager()
                                .unwrap()
                                .v_crew_list
                                .iter()
                                .find(|x| {
                                    map.map(
                                        unsafe { xc(**x).unwrap() }
                                            .blueprint
                                            .crew_name_long
                                            .to_str(),
                                    ) == event.old_name.as_str()
                                })
                                .copied()
                        });
                        if let Some(c) = c {
                            let crew = &mut app.gui_mut().unwrap().crew_screen;
                            let cc = crew
                                .crew_boxes
                                .iter()
                                .map(|x| unsafe { xm(*x).unwrap() })
                                .find(|x| !x.base.item.is_empty() && x.base.item.p_crew == c);
                            if let Some(cc) = cc {
                                if cc.b_show_rename {
                                    for b in crew.crew_boxes.iter() {
                                        let b = unsafe { xm(*b).unwrap() };
                                        b.delete_button.base.b_hover = false;
                                        b.rename_button.base.b_hover = false;
                                    }
                                    cc.rename_button.base.b_hover = true;
                                    unsafe {
                                        crew.base.vtable().mouse_click(
                                            ptr::addr_of_mut!(crew.base),
                                            0,
                                            0,
                                        );
                                    }
                                    if cc.name_input.b_active {
                                        let old = cc
                                            .name_input
                                            .text
                                            .iter()
                                            .filter_map(|x| char::from_u32(*x as u32))
                                            .collect::<String>();

                                        unsafe {
                                            app.base.vtable().on_text_event(
                                                ptr::addr_of_mut!(app.base),
                                                bindings::TextEvent::Clear,
                                            );
                                        }
                                        for char in event.name.chars() {
                                            unsafe {
                                                app.base.vtable().on_text_input(
                                                    ptr::addr_of_mut!(app.base),
                                                    char as i32,
                                                );
                                            }
                                        }
                                        let crew = &mut app.gui_mut().unwrap().crew_screen;
                                        unsafe {
                                            crew.base.vtable().mouse_click(
                                                ptr::addr_of_mut!(crew.base),
                                                0,
                                                0,
                                            );
                                        }
                                        Ok(Cow::from(format!(
                                            "renamed the crew member, old name is {old:?}, new name is {:?}",
                                            event.name
                                        ))
                                        .into())
                                    } else {
                                        Err(Cow::from(
                                        "couldn't rename the crew member, this is a bug in the mod",
                                    )
                                    .into())
                                    }
                                } else {
                                    Err(Cow::from("can't rename this crew member").into())
                                }
                            } else {
                                Err(Cow::from("can't rename this crew member").into())
                            }
                        } else {
                            let names = IdMap::with(|map| {
                                app.gui()
                                    .unwrap()
                                    .ship_manager()
                                    .unwrap()
                                    .v_crew_list
                                    .iter()
                                    .map(|x| {
                                        map.map(
                                            unsafe { xc(*x).unwrap() }
                                                .blueprint
                                                .crew_name_long
                                                .to_str(),
                                        )
                                    })
                                    .collect::<Vec<_>>()
                            });
                            Err(Cow::from(format!(
                                "this crew member doesn't exist, current crew members: {}",
                                serde_json::to_string(&names).unwrap()
                            ))
                            .into())
                        }
                    }
                } else {
                    Err(Cow::from("can't rename crew at this time").into())
                }
            }
            FtlActions::StartGame(event) => {
                if self.actions.valid(&event) {
                    let b = &mut app.menu.ship_builder;
                    for b in b.v_crew_boxes.iter() {
                        let b = unsafe { xm(*b).unwrap() };
                        b.customize_button.base.b_hover = false;
                    }
                    // force enable advanced edition
                    /*if b.advanced_on_button.base.b_active {
                        b.start_button.base.b_hover = false;
                        b.hard_button.base.b_hover = false;
                        b.easy_button.base.b_hover = false;
                        b.normal_button.base.b_hover = false;
                        b.rename_button.base.b_hover = false;
                        b.left_button.base.b_hover = false;
                        b.right_button.base.b_hover = false;
                        b.show_button.base.b_hover = false;
                        b.list_button.base.b_hover = false;
                        b.type_a.base.b_hover = false;
                        b.type_b.base.b_hover = false;
                        b.type_c.base.b_hover = false;
                        b.advanced_off_button.base.b_hover = false;
                        b.advanced_on_button.base.b_hover = true;
                        unsafe {
                            app.base
                                .vtable()
                                .on_l_button_down(ptr::addr_of_mut!(app.base), 0, 0);
                        }
                    }*/
                    // force enable easy mode
                    if b.easy_button.base.b_active {
                        b.start_button.base.b_hover = false;
                        b.hard_button.base.b_hover = false;
                        b.easy_button.base.b_hover = true;
                        unsafe {
                            app.base
                                .vtable()
                                .on_l_button_down(ptr::addr_of_mut!(app.base), 0, 0);
                        }
                    }
                    if b.start_button.base.b_active {
                        b.start_button.base.b_hover = true;
                        unsafe {
                            app.base
                                .vtable()
                                .on_l_button_down(ptr::addr_of_mut!(app.base), 0, 0);
                        }
                        Ok(Cow::from("successfully started the game").into())
                    } else {
                        Err(Cow::from("couldn't start the game, this is a bug in the mod").into())
                    }
                } else {
                    Err(Cow::from("can't start the game at this time").into())
                }
            }
            /*
             * ship selection, to select a ship:
             * b.ship_select.b_open = true;
             * b.ship_select.tutorial.b_open = false;
             * b.ship_select.type_a.b_hover = false;
             * b.ship_select.type_b.b_hover = false;
             * b.ship_select.type_c.b_hover = false;
             * b.ship_select.selected_ship = 0..=9;
             * b.ship_select.current_type = 0..=2;
             */
            FtlActions::MainMenu(event) => {
                if self.actions.valid(&event) && app.gui().unwrap().game_over_screen.base.b_open {
                    let sc = &mut app.gui_mut().unwrap().game_over_screen;
                    // 2: quit, 5: main menu, 6: hangar, 7: stats
                    sc.command = 5;
                    /*unsafe {
                        app.base
                            .vtable()
                            .on_key_down(ptr::addr_of_mut!(app.base), 27);
                    }*/
                    Ok(Cow::from("entered the main menu").into())
                } else {
                    Err(Cow::from("can't enter the main menu at this time").into())
                }
            }
            FtlActions::Choose1(_)
            | FtlActions::Choose2(_)
            | FtlActions::Choose3(_)
            | FtlActions::Choose4(_)
            | FtlActions::Choose5(_)
            | FtlActions::Choose6(_)
            | FtlActions::Choose7(_)
            | FtlActions::Choose8(_)
            | FtlActions::Choose9(_) => {
                let (index, valid) = match action {
                    FtlActions::Choose1(event) => (0usize, self.actions.valid(&event)),
                    FtlActions::Choose2(event) => (1usize, self.actions.valid(&event)),
                    FtlActions::Choose3(event) => (2usize, self.actions.valid(&event)),
                    FtlActions::Choose4(event) => (3usize, self.actions.valid(&event)),
                    FtlActions::Choose5(event) => (4usize, self.actions.valid(&event)),
                    FtlActions::Choose6(event) => (5usize, self.actions.valid(&event)),
                    FtlActions::Choose7(event) => (6usize, self.actions.valid(&event)),
                    FtlActions::Choose8(event) => (7usize, self.actions.valid(&event)),
                    FtlActions::Choose9(event) => (8usize, self.actions.valid(&event)),
                    _ => panic!(),
                };
                if valid {
                    if let Some(b) = app.gui().unwrap().choice_box.choices.get(index) {
                        if b.type_ == 1 {
                            Err(Cow::from(format!("option {index} requirements not met, can't choose this! Please pick a different option.")).into())
                        } else {
                            app.gui_mut().unwrap().choice_box.selected_choice = index as i32;
                            unsafe {
                                app.gui().unwrap().choice_box.base.vtable().close(
                                    ptr::addr_of_mut!(app.gui_mut().unwrap().choice_box.base),
                                );
                            }
                            Ok(Cow::from(format!(
                                "option {} chosen.{}",
                                index,
                                resource_event_str(
                                    &app.gui()
                                        .unwrap()
                                        .choice_box
                                        .choices
                                        .get(index)
                                        .unwrap()
                                        .rewards,
                                    app.gui().unwrap().ship_manager().unwrap()
                                )
                            ))
                            .into())
                        }
                    } else {
                        Err(Cow::from("invalid choice").into())
                    }
                } else {
                    Err(Cow::from("can't choose an event option at the time").into())
                }
            }
            FtlActions::IncreaseSystemPower(_) | FtlActions::DecreaseSystemPower(_) => {
                let (valid, system, amount, increase) = match action {
                    FtlActions::IncreaseSystemPower(event) => {
                        (self.actions.valid(&event), event.system, event.amount, true)
                    }
                    FtlActions::DecreaseSystemPower(event) => (
                        self.actions.valid(&event),
                        event.system,
                        event.amount,
                        false,
                    ),
                    _ => unreachable!(),
                };
                if valid {
                    let system = IdMap::with(|map| {
                        app.gui_mut()
                            .unwrap()
                            .ship_manager_mut()
                            .unwrap()
                            .systems_mut()
                            .find(|x| {
                                map.map(
                                    System::from_id(x.i_system_type)
                                        .unwrap()
                                        .blueprint()
                                        .unwrap()
                                        .title
                                        .to_str()
                                        .into(),
                                ) == system
                            })
                    });
                    if let Some(system) = system {
                        if increase {
                            if system.i_lock_count == -1 || system.i_lock_count > 0 {
                                Err(Cow::from("the system can't be controlled at the time").into())
                            } else if system.i_hack_effect > 1 {
                                Err(Cow::from(
                                    "the system has been hacked and can't be controlled at the time",
                                )
                                .into())
                            } else if !system.b_needs_power {
                                Err(Cow::from("the system does not require any power").into())
                            } else if unsafe {
                                system.vtable().force_increase_power(
                                    std::ptr::addr_of_mut!(*system),
                                    amount.into(),
                                )
                            } {
                                system.last_user_power = system.i_battery_power
                                    + system.i_bonus_power
                                    + system.power_state.first;
                                Ok(Cow::from(format!(
                                    "system power successfully increased to {}/{}, reactor state: {}",
                                    system.power_state.first, system.power_state.second, reactor_state(system.i_ship_id),
                                ))
                                .into())
                            } else {
                                Err(Cow::from(
                                    "failed to increase power; either the system is already at max power or you must power down other systems first",
                                )
                                .into())
                            }
                        } else if !system.b_needs_power {
                            Err(Cow::from("the system can not use any power").into())
                        } else if system.i_lock_count == -1 || system.i_lock_count > 0 {
                            Err(Cow::from("the system can't be controlled at the time").into())
                        } else if system.i_hack_effect > 1 {
                            Err(Cow::from(
                                "the system has been hacked and can't be controlled at the time",
                            )
                            .into())
                        } else if unsafe {
                            system.vtable().force_decrease_power(
                                std::ptr::addr_of_mut!(*system),
                                amount.into(),
                            )
                        } {
                            system.last_user_power = system.i_battery_power
                                + system.i_bonus_power
                                + system.power_state.first;
                            Ok(Cow::from(format!(
                                "system power successfully decreased to {}/{}, reactor state: {}",
                                system.power_state.first,
                                system.power_state.second,
                                reactor_state(system.i_ship_id)
                            ))
                            .into())
                        } else {
                            Err(
                                Cow::from("can't decrease the system's power, it's probably already powered down")
                                    .into(),
                            )
                        }
                    } else {
                        let systems = IdMap::with(|map| {
                            app.gui()
                                .unwrap()
                                .ship_manager()
                                .unwrap()
                                .systems()
                                .filter_map(|x| {
                                    System::from_id(x.i_system_type).map(|x| {
                                        map.map(x.blueprint().unwrap().title.to_str().into())
                                    })
                                })
                                .collect::<Vec<_>>()
                        });
                        Err(Cow::from(format!(
                            "this system doesn't exist, current systems: {}",
                            serde_json::to_string(&systems).unwrap()
                        ))
                        .into())
                    }
                } else if increase {
                    Err(Cow::from("can't increase systems' power at the time").into())
                } else {
                    Err(Cow::from("can't decrease systems' power at the time").into())
                }
            }
            FtlActions::SetWeaponTargets(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't target weapons at the time").into())
                } else {
                    let gui = app.gui().unwrap();
                    let ship_manager = gui.ship_manager().unwrap();
                    let cc = &gui.combat_control;
                    let b = IdMap::with(|map| {
                        cc.weap_control
                            .base
                            .boxes
                            .iter()
                            .map(|x| x.cast::<bindings::WeaponBox>())
                            .find(|x| {
                                unsafe { xc(*x).unwrap() }.weapon().is_some_and(|x| {
                                    x.blueprint().is_some_and(|x| {
                                        map.map(x.desc.title.to_str()) == event.weapon_name
                                    })
                                })
                            })
                    });
                    if let Some(b) = b {
                        let weapon = unsafe { xm(b).unwrap() }.weapon_mut().unwrap();
                        if event.target_ship == TargetShip::Player
                            && !weapon.blueprint().unwrap().can_target_self()
                        {
                            Err(Cow::from("can't target the player ship with this weapon").into())
                        } else if event.target_ship == TargetShip::Enemy
                            && gui.combat_control.current_target.is_null()
                        {
                            Err(Cow::from("can't target the enemy because there's no enemy").into())
                        } else if weapon.num_targets_required() == 0 {
                            Err(
                                Cow::from("this weapon currently doesn't accept any targets")
                                    .into(),
                            )
                        } else if (weapon.num_targets_required() as usize)
                            != event.target_room_ids.len()
                        {
                            Err(Cow::from(format!(
                                "this weapon currently requires {} targets, not {}",
                                weapon.num_targets_required(),
                                event.target_room_ids.len()
                            ))
                            .into())
                        } else if !weapon.powered {
                            Err(Cow::from("this weapon isn't currently powered").into())
                        } else {
                            let target_ship = match event.target_ship {
                                TargetShip::Player => ship_manager,
                                TargetShip::Enemy => gui
                                    .combat_control
                                    .current_target()
                                    .unwrap()
                                    .ship_manager()
                                    .unwrap(),
                            };
                            match event
                                .target_room_ids
                                .into_iter()
                                .map(|room| {
                                    if let Some(room) = target_ship
                                        .ship
                                        .v_room_list
                                        .iter()
                                        .map(|x| unsafe { xc(*x).unwrap() })
                                        .find(|x| x.i_room_id == i32::from(room))
                                    {
                                        let rect = &room.rect;
                                        Ok((
                                            (rect.x + rect.w / 2) as f32,
                                            (rect.y + rect.h / 2) as f32,
                                        ))
                                    } else {
                                        Err(Cow::from(format!(
                                            "room {} not found in this ship",
                                            room
                                        ))
                                        .into())
                                    }
                                })
                                .collect::<Result<Vec<_>, _>>()
                            {
                                Ok(points) => {
                                    weapon.auto_firing = event.autofire;
                                    let mut v1 = bindings::Vector::with_capacity(points.len());
                                    let mut v2 = bindings::Vector::with_capacity(points.len());
                                    for (x, y) in points {
                                        v1.push(bindings::Pointf { x, y });
                                        v2.push(bindings::Pointf { x, y });
                                    }
                                    weapon.targets = v1;
                                    weapon.last_targets = v2;
                                    weapon.target_id = target_ship.i_ship_id;
                                    weapon.fire_when_ready = true;
                                    if target_ship.i_ship_id == 0 {
                                        weapon.current_firing_angle = 0.0;
                                    } else {
                                        weapon.current_firing_angle = 270.0;
                                    }
                                    weapon.current_entry_angle =
                                        rand::thread_rng().gen_range(0..360) as f32;
                                    Ok(Cow::from("successfully targeted the weapon").into())
                                }
                                Err(err) => Err(err),
                            }
                        }
                    } else {
                        let weapons: Vec<_> = IdMap::with(|map| {
                            cc.weap_control
                                .base
                                .boxes
                                .iter()
                                .map(|x| x.cast::<bindings::WeaponBox>())
                                .map(|x| unsafe { xc(x).unwrap() })
                                .filter_map(|x| {
                                    x.weapon().and_then(|x| {
                                        x.blueprint().map(|x| map.map(x.desc.title.to_str()))
                                    })
                                })
                                .collect()
                        });
                        Err(Cow::from(format!(
                            "no weapon with this name, available weapons: {}",
                            serde_json::to_string(&weapons).unwrap()
                        ))
                        .into())
                    }
                }
            }
            FtlActions::ActivateDrone(_) | FtlActions::DeactivateDrone(_) => {
                let (valid, drone_name, activate) = match action {
                    FtlActions::ActivateDrone(event) => {
                        (self.actions.valid(&event), event.drone_name, true)
                    }
                    FtlActions::DeactivateDrone(event) => {
                        (self.actions.valid(&event), event.drone_name, false)
                    }
                    _ => unreachable!(),
                };
                if !valid {
                    Err(Cow::from("can't control a drone at the time").into())
                } else {
                    let ship_manager = app.gui_mut().unwrap().ship_manager_mut().unwrap();
                    let drone_system = ship_manager.drone_system().unwrap();
                    if drone_system.base.i_lock_count == -1 || drone_system.base.i_lock_count > 0 {
                        Err(Cow::from("the drone system can't be controlled at the time").into())
                    } else if drone_system.base.i_hack_effect > 1 {
                        Err(Cow::from(
                            "the drone system has been hacked and can't be controlled at the time",
                        )
                        .into())
                    } else {
                        let cc = &app.gui().unwrap().combat_control;
                        let b = IdMap::with(|map| {
                            cc.drone_control
                                .base
                                .boxes
                                .iter()
                                .map(|x| x.cast::<bindings::DroneBox>())
                                .map(|x| unsafe { xc(x).unwrap() })
                                .find(|x| {
                                    x.drone().is_some_and(|x| {
                                        x.blueprint().is_some_and(|x| {
                                            map.map(x.desc.title.to_str()) == drone_name
                                        })
                                    })
                                })
                        });
                        if let Some(b) = b {
                            let ship_manager = app.gui_mut().unwrap().ship_manager_mut().unwrap();
                            let drone_system = ship_manager.drone_system().unwrap();
                            let drone = b.drone().unwrap();
                            if activate {
                                let was_deployed = !drone.deployed;
                                if drone.powered {
                                    Err(Cow::from("this drone is already powered").into())
                                } else if !drone.deployed && ship_manager.drone_count() == 0 {
                                    Err(Cow::from(
                                        "you have no drone parts left to deploy this drone",
                                    )
                                    .into())
                                } else if !drone.deployed
                                    && !unsafe { drone.vtable().can_be_deployed(b.p_drone) }
                                {
                                    Err(Cow::from("the drone can't currently be deployed, probably because there's no enemy ship").into())
                                } else if drone.destroyed_timer > 0.0 {
                                    Err(Cow::from(
                                        "the drone is still rebuilding and can't be deployed at the moment",
                                    )
                                    .into())
                                } else if drone_system.base.available_power()
                                    < drone.required_power()
                                {
                                    // not enough power
                                    if drone_system.base.power_max() < drone.required_power() {
                                        Err(Cow::from(
                                    format!("the drone system is currently at {}/{} power usage, while the drone requires {} power, you could try upgrading the system to increase max power", drone_system.base.effective_power(), drone_system.base.max_power(), drone.required_power()),
                                )
                                .into())
                                    } else if drone_system.base.power_state.second
                                        - drone_system.base.power_state.first
                                        >= drone.required_power()
                                        && drone_system.base.damage() > 0
                                    {
                                        Err(Cow::from(
                                    format!("the drone system is currently at {}/{} power usage, while the drone requires {} power, you could try repairing the system to increase max power", drone_system.base.effective_power(), drone_system.base.max_power(), drone.required_power()),
                                )
                                .into())
                                    } else {
                                        Err(Cow::from(
                                    format!("the drone system is currently at {}/{} power usage, while the drone requires {} power, you could try powering down other drones", drone_system.base.effective_power(), drone_system.base.max_power(), drone.required_power()),
                                )
                                .into())
                                    }
                                } else if unsafe {
                                    ship_manager.power_drone(b.p_drone, 1, true, false)
                                } {
                                    if was_deployed {
                                        Ok(Cow::from("successfully powered the drone").into())
                                    } else {
                                        Ok(Cow::from("successfully deployed the drone").into())
                                    }
                                } else {
                                    Err(Cow::from("failed to power the drone").into())
                                }
                            } else if unsafe { ship_manager.depower_drone(b.p_drone, true) } {
                                Ok(Cow::from("successfully depowered the drone").into())
                            } else {
                                Err(Cow::from(
                                    "couldn't depower the drone, it's probably already depowered",
                                )
                                .into())
                            }
                        } else {
                            let drones: Vec<_> = IdMap::with(|map| {
                                cc.drone_control
                                    .base
                                    .boxes
                                    .iter()
                                    .map(|x| x.cast::<bindings::DroneBox>())
                                    .map(|x| unsafe { xc(x).unwrap() })
                                    .filter_map(|x| {
                                        x.drone().and_then(|x| {
                                            x.blueprint().map(|x| map.map(x.desc.title.to_str()))
                                        })
                                    })
                                    .collect()
                            });
                            Err(Cow::from(format!(
                                "no drone with this name, available drones: {}",
                                serde_json::to_string(&drones).unwrap()
                            ))
                            .into())
                        }
                    }
                }
            }
            FtlActions::ActivateWeapon(_) | FtlActions::DeactivateWeapon(_) => {
                let (valid, weapon_name, activate) = match action {
                    FtlActions::ActivateWeapon(event) => {
                        (self.actions.valid(&event), event.weapon_name, true)
                    }
                    FtlActions::DeactivateWeapon(event) => {
                        (self.actions.valid(&event), event.weapon_name, false)
                    }
                    _ => unreachable!(),
                };
                if !valid {
                    Err(Cow::from("can't control a weapon at the time").into())
                } else {
                    let ship_manager = app.gui_mut().unwrap().ship_manager_mut().unwrap();
                    let weapon_system = ship_manager.weapon_system().unwrap();
                    if weapon_system.base.i_lock_count == -1 || weapon_system.base.i_lock_count > 0
                    {
                        Err(Cow::from("the weapon system can't be controlled at the time").into())
                    } else if weapon_system.base.i_hack_effect > 1 {
                        Err(Cow::from(
                            "the weapon system has been hacked and can't be controlled at the time",
                        )
                        .into())
                    } else {
                        let cc = &app.gui().unwrap().combat_control;
                        let b = IdMap::with(|map| {
                            cc.weap_control
                                .base
                                .boxes
                                .iter()
                                .map(|x| x.cast::<bindings::WeaponBox>())
                                .map(|x| unsafe { xc(x).unwrap() })
                                .find(|x| {
                                    x.weapon().is_some_and(|x| {
                                        x.blueprint().is_some_and(|x| {
                                            map.map(x.desc.title.to_str()) == weapon_name
                                        })
                                    })
                                })
                        });
                        if let Some(b) = b {
                            let ship_manager = app.gui_mut().unwrap().ship_manager_mut().unwrap();
                            let weapon_system = ship_manager.weapon_system().unwrap();
                            let weapon = b.weapon().unwrap();
                            if activate {
                                if weapon.powered {
                                    Err(Cow::from("this weapon is already powered").into())
                                } else if weapon.blueprint().unwrap().missiles != 0
                                    && ship_manager.missile_count() == 0
                                {
                                    Err(Cow::from("you have no missiles left to use this weapon")
                                        .into())
                                } else if weapon_system.base.available_power()
                                    < weapon.required_power - weapon.i_bonus_power
                                {
                                    // not enough power
                                    if weapon_system.base.power_max()
                                        < weapon.required_power - weapon.i_bonus_power
                                    {
                                        Err(Cow::from(
                                            format!("the weapon system is currently at {}/{} power usage, while the weapon requires {} power, you could try upgrading the system to increase max power", weapon_system.base.effective_power(), weapon_system.base.max_power(), weapon.required_power - weapon.i_bonus_power),
                                        )
                                        .into())
                                    } else if weapon_system.base.power_state.second
                                        - weapon_system.base.power_state.first
                                        >= weapon.required_power
                                        && weapon_system.base.damage() > 0
                                    {
                                        Err(Cow::from(
                                            format!("the weapon system is currently at {}/{} power usage, while the weapon requires {} power, you could try repairing the system to increase max power", weapon_system.base.effective_power(), weapon_system.base.max_power(), weapon.required_power - weapon.i_bonus_power),
                                        )
                                        .into())
                                    } else {
                                        Err(Cow::from(
                                            format!("the weapon system is currently at {}/{} power usage, while the weapon requires {} power, you could try powering down other weapons", weapon_system.base.effective_power(), weapon_system.base.max_power(), weapon.required_power - weapon.i_bonus_power),
                                        )
                                        .into())
                                    }
                                } else if unsafe {
                                    ship_manager.power_weapon(b.p_weapon, true, false)
                                } {
                                    Ok(Cow::from("successfully powered the weapon").into())
                                } else {
                                    Err(Cow::from("failed to power the weapon").into())
                                }
                            } else if unsafe { ship_manager.depower_weapon(b.p_weapon, true) } {
                                Ok(Cow::from("successfully depowered the weapon").into())
                            } else {
                                Err(Cow::from(
                                    "couldn't depower the weapon, it's probably already depowered",
                                )
                                .into())
                            }
                        } else {
                            let weapons: Vec<_> = IdMap::with(|map| {
                                cc.weap_control
                                    .base
                                    .boxes
                                    .iter()
                                    .map(|x| x.cast::<bindings::WeaponBox>())
                                    .map(|x| unsafe { xc(x).unwrap() })
                                    .filter_map(|x| {
                                        x.weapon().and_then(|x| {
                                            x.blueprint().map(|x| map.map(x.desc.title.to_str()))
                                        })
                                    })
                                    .collect()
                            });
                            Err(Cow::from(format!(
                                "no weapon with this name, available weapons: {}",
                                serde_json::to_string(&weapons).unwrap()
                            ))
                            .into())
                        }
                    }
                }
            }
            FtlActions::HackSystem(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't launch a hacking drone at the time").into())
                } else {
                    let gui = app.gui_mut().unwrap();
                    if let Some(target) = gui.combat_control.current_target_mut() {
                        let target = target.ship_manager_mut().unwrap();
                        let system = IdMap::with(|map| {
                            target.systems_mut().find(|x| {
                                map.map(
                                    System::from_id(x.i_system_type)
                                        .unwrap()
                                        .blueprint()
                                        .unwrap()
                                        .title
                                        .to_str()
                                        .into(),
                                ) == event.system
                            })
                        });
                        if let Some(system) = system {
                            let system = ptr::addr_of_mut!(*system);
                            let drone_count = gui.ship_manager().unwrap().drone_count();
                            let jumping = gui.ship_manager().unwrap().b_jumping;
                            let dying = gui.ship_manager().unwrap().b_destroyed;
                            let hack = gui
                                .ship_manager_mut()
                                .unwrap()
                                .hacking_system_mut()
                                .unwrap();
                            if hack.b_blocked {
                                Err(Cow::from("can't hack a ship with Zoltan super shields").into())
                            } else if jumping {
                                Err(Cow::from("currently jumping, not launching the drone").into())
                            } else if dying {
                                Err(Cow::from("currently dying, not launching the drone").into())
                            } else if hack.base.i_lock_count == -1 || hack.base.i_lock_count > 0 {
                                Err(
                                    Cow::from("the hacking system can't be controlled at the time")
                                        .into(),
                                )
                            } else if hack.base.i_hack_effect > 1 {
                                Err(Cow::from(
                                    "the hacking system has been hacked and can't be controlled at the time",
                                )
                                .into())
                            } else if !hack.b_can_hack {
                                Err(
                                    Cow::from("the hacking system can't be activated at the time")
                                        .into(),
                                )
                            } else if hack.b_hacking {
                                Err(Cow::from("the hacking system has already been activated")
                                    .into())
                            } else if !hack.base.functioning() {
                                Err(Cow::from("the hacking system is not powered at the moment")
                                    .into())
                            } else if drone_count == 0 {
                                Err(Cow::from(
                                    "you need to have a drone part to launch a hacking drone",
                                )
                                .into())
                            } else {
                                hack.queued_system = system;
                                hack.b_armed = false;
                                Ok(Cow::from("successfully launched a drone").into())
                            }
                        } else {
                            let systems = IdMap::with(|map| {
                                target
                                    .systems()
                                    .map(|x| {
                                        map.map(
                                            System::from_id(x.i_system_type)
                                                .unwrap()
                                                .blueprint()
                                                .unwrap()
                                                .title
                                                .to_str()
                                                .into(),
                                        )
                                    })
                                    .collect::<Vec<_>>()
                            });
                            Err(Cow::from(format!(
                                "the enemy ship doesn't have this system, available systems: {}",
                                serde_json::to_string(&systems).unwrap()
                            ))
                            .into())
                        }
                    } else {
                        Err(Cow::from("can't hack the enemy because there's no enemy").into())
                    }
                }
            }
            FtlActions::MindControl(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't mind control at the time").into())
                } else {
                    let gui = app.gui_mut().unwrap();
                    let mind = gui.ship_manager_mut().unwrap().mind_system_mut().unwrap();
                    if mind.b_blocked {
                        Err(Cow::from(
                            "mind control is blocked by the enemy ship's Zoltan super shields",
                        )
                        .into())
                    } else if mind.base.i_lock_count == -1 || mind.base.i_lock_count > 0 {
                        Err(
                            Cow::from("the mind control system can't be controlled at the time")
                                .into(),
                        )
                    } else if mind.base.i_hack_effect > 1 {
                        Err(Cow::from(
                            "the mind control system has been hacked and can't be controlled at the time",
                        )
                        .into())
                    } else if event.target_ship == TargetShip::Player
                        && gui
                            .target_self_with_mind_control_error(event.target_room_id.into())
                            .is_some()
                    {
                        Err(Cow::from(
                            gui.target_self_with_mind_control_error(event.target_room_id.into())
                                .unwrap(),
                        )
                        .into())
                    } else if event.target_ship == TargetShip::Enemy
                        && gui.combat_control.current_target.is_null()
                    {
                        Err(Cow::from("there's no enemy ship at the moment").into())
                    } else if !gui
                        .ship_manager_mut()
                        .unwrap()
                        .mind_system_mut()
                        .unwrap()
                        .base
                        .functioning()
                    {
                        Err(
                            Cow::from("the mind control system is not powered at the moment")
                                .into(),
                        )
                    } else {
                        let target_ship = match event.target_ship {
                            TargetShip::Player => gui.ship_manager().unwrap(),
                            TargetShip::Enemy => gui
                                .combat_control
                                .current_target()
                                .unwrap()
                                .ship_manager()
                                .unwrap(),
                        };
                        let room = target_ship
                            .ship
                            .v_room_list
                            .iter()
                            .map(|x| unsafe { xc(*x).unwrap() })
                            .find(|x| x.i_room_id == i32::from(event.target_room_id));
                        if let Some(room) = room {
                            let ship_id = target_ship.i_ship_id;
                            let c = target_ship
                                .v_crew_list
                                .iter()
                                .copied()
                                .filter(|x| unsafe { xc(*x).unwrap() }.i_room_id == room.i_room_id)
                                .collect::<Vec<_>>();
                            if c.is_empty() {
                                Err(Cow::from(format!(
                                    "no crew in enemy ship's room {}",
                                    event.target_room_id
                                ))
                                .into())
                            } else {
                                let mind =
                                    gui.ship_manager_mut().unwrap().mind_system_mut().unwrap();
                                mind.i_queued_target = room.i_room_id;
                                mind.i_queued_ship = ship_id;
                                let mut b = bindings::Vector::with_capacity(c.len());
                                for x in c {
                                    b.push(x);
                                }
                                mind.queued_crew = b;
                                Ok(Cow::from("successfully activated mind control").into())
                            }
                        } else {
                            Err(Cow::from(format!(
                                "room {} not found in this ship",
                                event.target_room_id
                            ))
                            .into())
                        }
                    }
                }
            }
            FtlActions::ActivateHacking(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't use the hacking drone at the time").into())
                } else {
                    let gui = app.gui_mut().unwrap();
                    if gui.combat_control.current_target.is_null() {
                        Err(Cow::from("can't hack the enemy because there's no enemy").into())
                    } else {
                        let sys = gui
                            .ship_manager_mut()
                            .unwrap()
                            .hacking_system_mut()
                            .unwrap();
                        if !sys.b_hacking {
                            Err(Cow::from("the hacking system is inactive").into())
                        } else if sys.base.effective_power() == 0 {
                            Err(Cow::from("the hacking system is unpowered").into())
                        } else if sys.current_system().is_none() {
                            Err(Cow::from("the hacking drone hasn't been launched yet").into())
                        } else if sys
                            .current_system()
                            .is_some_and(|x| x.health_state.first == 0)
                        {
                            Err(Cow::from(
                                "the target system is destroyed so its function can't be disrupted",
                            )
                            .into())
                        } else if !sys.drone.arrived {
                            Err(
                                Cow::from("the hacking drone hasn't arrived to the enemy ship yet")
                                    .into(),
                            )
                        } else if sys.b_blocked {
                            Err(Cow::from("can't hack a ship with Zoltan super shields").into())
                        } else if sys.base.i_lock_count == -1 || sys.base.i_lock_count > 0 {
                            Err(
                                Cow::from("the hacking system can't be controlled at the time")
                                    .into(),
                            )
                        } else if sys.base.i_hack_effect > 1 {
                            Err(Cow::from(
                                    "the hacking system has been hacked and can't be controlled at the time",
                                )
                                .into())
                        } else {
                            let mut ret = Err(Cow::from(
                                    "the hacking system button has not been found, this is probably a bug in the mod",
                                )
                                .into());
                            let sys = ptr::addr_of_mut!(*sys.base.deref_mut());
                            for b in gui.sys_control.sys_boxes.iter() {
                                if unsafe { xc(*b).unwrap() }.p_system == sys {
                                    let b = b.cast::<bindings::HackBox>();
                                    let b = unsafe { xm(b).unwrap() };
                                    if !b.current_button().unwrap().base.b_active {
                                        continue;
                                    }
                                    b.current_button_mut().unwrap().base.b_hover = true;
                                    b.base.base.mouse_hover = false;
                                    unsafe {
                                        b.base
                                            .base
                                            .vtable()
                                            .mouse_click(ptr::addr_of_mut!(b.base.base), false);
                                    }
                                    ret = Ok(Cow::from("successfully initiated hacking").into());
                                    break;
                                }
                            }
                            ret
                        }
                    }
                }
            }
            FtlActions::ActivateBattery(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't use the battery subsystem at the time").into())
                } else {
                    let gui = app.gui_mut().unwrap();
                    let sys = gui
                        .ship_manager_mut()
                        .unwrap()
                        .battery_system_mut()
                        .unwrap();
                    if sys.b_turned_on {
                        Err(Cow::from("the batery system is already turned on").into())
                    } else if sys.base.i_lock_count == -1 || sys.base.i_lock_count > 0 {
                        Err(
                            Cow::from("the battery subsystem can't be controlled at the time")
                                .into(),
                        )
                    } else if sys.base.i_hack_effect > 1 {
                        Err(Cow::from(
                            "the battery subsystem has been hacked and can't be controlled at the time",
                        )
                        .into())
                    } else {
                        let mut ret = Err(Cow::from(
                            "the battery subsystem button has not been found, this is probably a bug in the mod",
                        )
                        .into());
                        let sys = ptr::addr_of_mut!(*sys.base.deref_mut());
                        for b in gui.sys_control.sys_boxes.iter() {
                            if unsafe { xc(*b).unwrap() }.p_system == sys {
                                let b = b.cast::<bindings::BatteryBox>();
                                let b = unsafe { xm(b).unwrap() };
                                if !b.battery_button.base.b_active {
                                    continue;
                                }
                                b.battery_button.base.b_hover = true;
                                b.base.base.mouse_hover = false;
                                unsafe {
                                    b.base
                                        .base
                                        .vtable()
                                        .mouse_click(ptr::addr_of_mut!(b.base.base), false);
                                }
                                ret = Ok(Cow::from("successfully started the battery").into());
                                break;
                            }
                        }
                        ret
                    }
                }
            }
            FtlActions::ActivateCloaking(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't use the cloaking system at the time").into())
                } else {
                    let gui = app.gui_mut().unwrap();
                    let sys = gui.ship_manager_mut().unwrap().cloak_system_mut().unwrap();
                    if sys.b_turned_on {
                        Err(Cow::from("the cloaking system is already turned on").into())
                    } else if sys.base.i_lock_count == -1 || sys.base.i_lock_count > 0 {
                        Err(Cow::from("the cloaking system can't be controlled at the time").into())
                    } else if sys.base.i_hack_effect > 1 {
                        Err(Cow::from(
                            "the cloaking system has been hacked and can't be controlled at the time",
                        )
                        .into())
                    } else if !sys.base.functioning() {
                        Err(Cow::from("the cloaking system is not powered at the moment").into())
                    } else {
                        let mut ret = Err(Cow::from(
                            "the cloaking system button has not been found, this is probably a bug in the mod",
                        )
                        .into());
                        let sys = ptr::addr_of_mut!(*sys.base.deref_mut());
                        for b in gui.sys_control.sys_boxes.iter() {
                            if unsafe { xc(*b).unwrap() }.p_system == sys {
                                let b = b.cast::<bindings::CloakingBox>();
                                let b = unsafe { xm(b).unwrap() };
                                if !b.current_button().unwrap().base.b_active {
                                    continue;
                                }
                                b.current_button_mut().unwrap().base.b_hover = true;
                                b.base.base.mouse_hover = false;
                                unsafe {
                                    b.base
                                        .base
                                        .vtable()
                                        .mouse_click(ptr::addr_of_mut!(b.base.base), false);
                                }
                                assert!(
                                    gui.ship_manager()
                                        .unwrap()
                                        .cloak_system()
                                        .unwrap()
                                        .b_turned_on
                                );
                                ret = Ok(Cow::from("successfully initiated cloaking").into());
                                break;
                            }
                        }
                        ret
                    }
                }
            }
            FtlActions::TeleportSend(_) | FtlActions::TeleportReturn(_) => {
                let (valid, send, room) = match action {
                    FtlActions::TeleportSend(event) => {
                        (self.actions.valid(&event), true, event.target_room_id)
                    }
                    FtlActions::TeleportReturn(event) => {
                        (self.actions.valid(&event), false, event.source_room_id)
                    }
                    _ => unreachable!(),
                };
                if !valid {
                    Err(Cow::from("can't use the teleporter system at the time").into())
                } else {
                    let gui = app.gui_mut().unwrap();
                    let bypass = gui.equip_screen.has_augment("ZOLTAN_BYPASS");
                    let sys = gui
                        .ship_manager_mut()
                        .unwrap()
                        .teleport_system_mut()
                        .unwrap();
                    if send && !sys.b_can_send {
                        Err(Cow::from("the teleporter system can't send crew at the time, probably because there's no enemy ship").into())
                    } else if !send && !sys.b_can_receive {
                        Err(Cow::from("the teleporter system can't receive crew at the time, probably because there's no enemy ship").into())
                    } else if send && sys.b_super_shields && !bypass {
                        Err(Cow::from("can't teleport to a ship with Zoltan super shields").into())
                    } else if send && sys.i_prepared_crew == 0 {
                        Err(Cow::from("there's no crew to send in the teleporter room").into())
                    } else if sys.base.i_lock_count == -1 || sys.base.i_lock_count > 0 {
                        Err(
                            Cow::from("the teleporter system can't be controlled at the time")
                                .into(),
                        )
                    } else if sys.base.i_hack_effect > 1 {
                        Err(Cow::from(
                            "the teleporter system has been hacked and can't be controlled at the time",
                        )
                        .into())
                    } else if !sys.base.functioning() {
                        Err(Cow::from("the teleporter system is not powered at the moment").into())
                    } else {
                        gui.combat_control.teleport_command = bindings::Pair {
                            first: c_int::from(room),
                            second: if send { 1 } else { 2 },
                        };
                        Ok(Cow::from("queued the teleporter system command, it will only work if there's any crew to actually teleport").into())
                    }
                }
            }
            FtlActions::OpenDoors(_) | FtlActions::CloseDoors(_) => {
                let (valid, open, doors, air) = match action {
                    FtlActions::OpenDoors(event) => (
                        self.actions.valid(&event),
                        true,
                        event.door_ids,
                        event.include_airlocks,
                    ),
                    FtlActions::CloseDoors(event) => {
                        (self.actions.valid(&event), false, event.door_ids, true)
                    }
                    _ => unreachable!(),
                };

                if !valid {
                    Err(Cow::from("can't use the doors system at the time").into())
                } else {
                    let gui = app.gui_mut().unwrap();
                    let sys = gui
                        .ship_manager_mut()
                        .unwrap()
                        .system(System::Doors)
                        .unwrap();
                    if sys.i_lock_count == -1 || sys.i_lock_count > 0 {
                        Err(Cow::from("the doors system can't be controlled at the time").into())
                    } else if sys.i_hack_effect > 1 {
                        Err(Cow::from(
                            "the doors system has been hacked and can't be controlled at the time",
                        )
                        .into())
                    } else {
                        let ship = &gui.ship_manager().unwrap().ship;
                        let all_doors: BTreeMap<c_int, *mut Door> = if air {
                            ship.v_door_list
                                .iter()
                                .copied()
                                .map(|door| (unsafe { xc(door).unwrap() }.i_door_id, door))
                                .chain(
                                    ship.v_outer_airlocks
                                        .iter()
                                        .copied()
                                        .enumerate()
                                        .map(|(i, door)| (-(i as c_int + 1), door)),
                                )
                                .collect()
                        } else {
                            ship.v_door_list
                                .iter()
                                .copied()
                                .map(|door| (unsafe { xc(door).unwrap() }.i_door_id, door))
                                .collect()
                        };
                        match doors
                            .into_iter()
                            .map(|x| {
                                all_doors.get(&c_int::from(x)).copied().ok_or_else(|| {
                                    Err(Some(Cow::from(format!("door {x} not found"))))
                                })
                            })
                            .collect::<Result<Vec<_>, _>>()
                        {
                            Ok(doors) => {
                                if doors.is_empty() {
                                    for door in all_doors.into_values() {
                                        let door = unsafe { xm(door).unwrap() };
                                        if door.i_hacked <= 0 && door.b_open != open {
                                            if open {
                                                door.open();
                                            } else {
                                                door.close();
                                            }
                                        }
                                    }
                                }
                                let mut hacked = Vec::new();
                                for door in &doors {
                                    let door = unsafe { &**door };
                                    if door.i_hacked > 0 && door.b_open != open {
                                        hacked.push(door.i_door_id.to_string());
                                    }
                                }
                                if hacked.is_empty() {
                                    for door in doors {
                                        let door = unsafe { xm(door).unwrap() };
                                        if door.i_hacked <= 0 && door.b_open != open {
                                            if open {
                                                door.open();
                                            } else {
                                                door.close();
                                            }
                                        }
                                    }
                                    if open {
                                        Ok(Cow::from("successfully opened the doors").into())
                                    } else {
                                        Ok(Cow::from("successfully closed the doors").into())
                                    }
                                } else {
                                    Err(Cow::from(format!(
                                        "doors {} are hacked and can't be controlled",
                                        hacked.join(", ")
                                    ))
                                    .into())
                                }
                            }
                            Err(err) => err,
                        }
                    }
                }
            }
            FtlActions::PlanDoorRoute(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from(
                        "can't control doors at the time, so this action is useless anyway",
                    )
                    .into())
                } else {
                    let gui = app.gui().unwrap();
                    let ship = &gui.ship_manager().unwrap().ship;
                    let mut graph = ShipGraph::default();
                    for (i, door) in ship
                        .v_door_list
                        .iter()
                        .copied()
                        .map(|door| (unsafe { xc(door).unwrap() }.i_door_id, door))
                        .chain(
                            ship.v_outer_airlocks
                                .iter()
                                .copied()
                                .enumerate()
                                .map(|(i, door)| (-(i as c_int + 1), door)),
                        )
                    {
                        let door = unsafe { xc(door).unwrap() };
                        graph.add_door(i, door.i_room1, door.i_room2);
                    }
                    match graph
                        .shortest_path(event.first_room_id.into(), event.second_room_id.into())
                    {
                        Ok(doors) => Ok(Cow::from(format!(
                            "the shortest path between the rooms: [{}]",
                            doors
                                .into_iter()
                                .map(|x| x.to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        ))
                        .into()),
                        Err(err) => Err(err),
                    }
                }
            }
            FtlActions::MoveCrew(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't move crew members at the time").into())
                } else {
                    let actions::MoveCrew {
                        mut crew_member_names,
                        ship,
                        room_id,
                    } = event;
                    let gui = app.gui().unwrap();
                    let crew = &gui.ship_manager().unwrap().v_crew_list;
                    let crew_map: HashMap<_, _> = IdMap::with(|map| {
                        crew.iter()
                            .map(|c| {
                                (
                                    map.map(
                                        unsafe { xc(*c).unwrap() }
                                            .blueprint
                                            .crew_name_long
                                            .to_str(),
                                    ),
                                    c,
                                )
                            })
                            .collect()
                    });
                    crew_member_names.sort();
                    let mut ret = Ok(());
                    for x in crew_member_names.windows(2) {
                        if x[0] == x[1] {
                            ret = Err(Some(Cow::from(format!(
                                "duplicate crew member: {:?}",
                                x[0]
                            ))));
                            break;
                        }
                    }
                    match ret.and_then(|()| {
                        crew_member_names
                            .into_iter()
                            .map(|x| {
                                if let Some(c) = crew_map.get(x.as_str()) {
                                    Ok((x, **c))
                                } else {
                                    let names = IdMap::with(|map| {
                                        crew.iter()
                                            .map(|x| {
                                                map.map(
                                                    unsafe { xc(*x).unwrap() }
                                                        .blueprint
                                                        .crew_name_long
                                                        .to_str(),
                                                )
                                            })
                                            .collect::<Vec<_>>()
                                    });
                                    Err(Some(Cow::from(format!(
                                        "crew member {:?} doesn't exist, current crew members: {}",
                                        x,
                                        serde_json::to_string(&names).unwrap()
                                    ))))
                                }
                            })
                            .collect::<Result<Vec<_>, _>>()
                    }) {
                        Ok(crew) if crew.is_empty() => Err(Some(Cow::from(
                            "must specify at least 1 crew member to move",
                        ))),
                        Ok(crew) => {
                            let mut err = None;
                            let mut crew1 = Vec::new();
                            let mut ignore = Vec::new();
                            for (i, c0) in &crew {
                                let c = unsafe { xc(*c0).unwrap() };
                                if c.f_stun_time > 0.0
                                    && (c.x - c.current_slot.world_location.x as f32).abs() < 0.5
                                    && (c.y - c.current_slot.world_location.y as f32).abs() < 0.5
                                {
                                    err = Some(Some(Cow::from(format!(
                                        "the crew member {i:?} is stunned or something like that idk"
                                    ))));
                                    break;
                                }
                                if c.b_dead {
                                    err = Some(Some(Cow::from(format!(
                                        "the crew member {i:?} is currently dead"
                                    ))));
                                    break;
                                }
                                if c.b_mind_controlled {
                                    err = Some(Some(Cow::from(format!(
                                        "the crew member {i:?} is currently mind controlled so he won't listen to your orders"
                                    ))));
                                    break;
                                }
                                if !unsafe { c.vtable().get_controllable(*c0) } {
                                    err = Some(Some(Cow::from(format!(
                                        "the crew member {i:?} is a drone and can't be controlled"
                                    ))));
                                    break;
                                }
                                if c.current_slot.room_id == i32::from(room_id) {
                                    ignore.push(i.to_string());
                                    continue;
                                }
                                let ship1 = if c.i_ship_id == c.current_ship_id {
                                    TargetShip::Player
                                } else {
                                    TargetShip::Enemy
                                };
                                if ship1 != ship {
                                    err = Some(Some(Cow::from(format!(
                                        "crew member {i:?} is on a different ship"
                                    ))));
                                    break;
                                }
                                crew1.push((i, *c0));
                            }
                            let (target_ship, s) = match ship {
                                TargetShip::Enemy => (
                                    gui.combat_control.current_target().unwrap().ship_manager(),
                                    "enemy",
                                ),
                                TargetShip::Player => (gui.ship_manager(), "player"),
                            };
                            if let Some(err) = err {
                                Err(err)
                            } else if crew1.is_empty() {
                                Ok(Cow::from(
                                    "no crew to move, everyone already in the target room",
                                )
                                .into())
                            } else if let Some(room) = target_ship
                                .unwrap()
                                .ship
                                .v_room_list
                                .iter()
                                .map(|x| unsafe { xc(*x).unwrap() })
                                .find(|x| x.i_room_id == i32::from(room_id))
                            {
                                let intruder =
                                    unsafe { xc(crew1.first().unwrap().1).unwrap() }.intruder();
                                if (room.available_slots(intruder) as usize) < crew1.len() {
                                    Err(Some(Cow::from(format!(
                                        "room {room_id} only has {} available slots, while you request requires moving {} crew members to the room", room.available_slots(intruder), crew1.len()
                                    ))))
                                } else {
                                    let mut yes = Vec::new();
                                    let mut no = Vec::new();
                                    for (i, c) in crew1 {
                                        if unsafe { xm(c).unwrap() }.move_to_room(
                                            room_id.into(),
                                            -1,
                                            false,
                                        ) {
                                            yes.push(i.to_string());
                                        } else {
                                            no.push(i.to_string());
                                        }
                                    }
                                    let mut s = Vec::new();
                                    if !yes.is_empty() {
                                        s.push(format!(
                                            "successfully moved crew members [{}]",
                                            yes.join(", ")
                                        ));
                                    }
                                    if !no.is_empty() {
                                        s.push(format!(
                                            "couldn't move crew members [{}]",
                                            no.join(", ")
                                        ));
                                    }
                                    if !ignore.is_empty() {
                                        s.push(format!(
                                            "didn't have to move crew members [{}]",
                                            ignore.join(", ")
                                        ));
                                    }
                                    Ok(Cow::from(s.join("; ")).into())
                                }
                            } else {
                                Err(Some(Cow::from(format!(
                                    "room {room_id} not found on the {s} ship"
                                ))))
                            }
                        }
                        Err(err) => Err(err),
                    }
                }
            }
            FtlActions::Lockdown(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't lockdown rooms at the time").into())
                } else {
                    let actions::Lockdown {
                        crew_member_name: name,
                        ship,
                        room_id,
                    } = event;
                    let gui = app.gui().unwrap();
                    let crew = &gui.ship_manager().unwrap().v_crew_list;
                    let crew = IdMap::with(|map| {
                        crew.iter().copied().find(|c| {
                            map.map(unsafe { xc(*c).unwrap() }.blueprint.crew_name_long.to_str())
                                == name
                        })
                    });
                    match crew {
                        Some(c0) => {
                            let c = unsafe { xc(c0).unwrap() };
                            if c.b_dead {
                                Err(Some(Cow::from(format!(
                                    "the crew member {name:?} is currently dead"
                                ))))
                            } else if !unsafe { c.vtable().has_special_power(c0) } {
                                Err(Some(Cow::from(format!(
                                    "the crew member {name:?} is not a crystal and can't lockdown rooms"
                                ))))
                            } else if !unsafe { c.vtable().power_ready(c0) } {
                                Err(Some(Cow::from(format!(
                                    "the crew member {name:?}'s power is currently on a cooldown"
                                ))))
                            } else if (if c.i_ship_id == c.current_ship_id {
                                TargetShip::Player
                            } else {
                                TargetShip::Enemy
                            }) != ship
                            {
                                Err(Some(Cow::from(format!(
                                    "crew member {name:?} is on a different ship"
                                ))))
                            } else if c.current_slot.room_id != i32::from(room_id) {
                                Err(Some(Cow::from(format!(
                                    "crew member {name:?} is in a different room"
                                ))))
                            } else {
                                unsafe { c.vtable().activate_power(c0) }
                                Ok(Cow::from("successfully locked the room down").into())
                            }
                        }
                        None => {
                            let names = IdMap::with(|map| {
                                app.gui()
                                    .unwrap()
                                    .ship_manager()
                                    .unwrap()
                                    .v_crew_list
                                    .iter()
                                    .map(|x| {
                                        map.map(
                                            unsafe { xc(*x).unwrap() }
                                                .blueprint
                                                .crew_name_long
                                                .to_str(),
                                        )
                                    })
                                    .collect::<Vec<_>>()
                            });
                            Err(Some(Cow::from(format!(
                                "crew member {:?} doesn't exist, current crew members: {}",
                                name,
                                serde_json::to_string(&names).unwrap()
                            ))))
                        }
                    }
                }
            }
            FtlActions::SwapInventorySlots(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't swap inventory slots at the time").into())
                } else {
                    let gui = app.gui().unwrap();
                    let e = &gui.equip_screen;
                    let slots = [event.slot1, event.slot2].map(
                        |actions::InventorySlot {
                             r#type: kind,
                             index,
                         }| {
                            if index == 0 {
                                return Err(Some(Cow::from("indices start at 1")));
                            }
                            match kind {
                                InventorySlotType::Cargo => {
                                    let b = e.boxes::<bindings::EquipmentBox>();
                                    let index = usize::from(index - 1);
                                    b.get(index).copied().ok_or_else(|| {
                                        Some(Cow::from(format!(
                                            "there are only {} cargo slots",
                                            b.len()
                                        )))
                                    })
                                }
                                InventorySlotType::Weapon => {
                                    let b = e.boxes::<bindings::WeaponEquipBox>();
                                    let index = usize::from(index - 1);
                                    b.get(index).copied().ok_or_else(|| {
                                        Some(Cow::from(format!(
                                            "there are only {} weapon slots",
                                            b.len()
                                        )))
                                    })
                                }
                                InventorySlotType::Drone => {
                                    let b = e.boxes::<bindings::DroneEquipBox>();
                                    let index = usize::from(index - 1);
                                    b.get(index).copied().ok_or_else(|| {
                                        Some(Cow::from(format!(
                                            "there are only {} drone slots",
                                            b.len()
                                        )))
                                    })
                                }
                                InventorySlotType::Augmentation => {
                                    let b = e.boxes::<bindings::AugmentEquipBox>();
                                    let index = usize::from(index - 1);
                                    b.get(index).copied().ok_or_else(|| {
                                        Some(Cow::from(format!(
                                            "there are only {} augment slots",
                                            b.len()
                                        )))
                                    })
                                }
                                InventorySlotType::AugmentationOverCapacity => e
                                    .b_over_aug_capacity
                                    .then_some(e.over_aug_box.cast())
                                    .ok_or_else(|| {
                                        Some(Cow::from(format!(
                                            "there is only {} over-capacity augment slot",
                                            u8::from(e.b_over_aug_capacity)
                                        )))
                                    }),
                                InventorySlotType::OverCapacity => e
                                    .b_over_capacity
                                    .then_some(e.overcapacity_box)
                                    .ok_or_else(|| {
                                        Some(Cow::from(format!(
                                            "there is only {} over-capacity slot",
                                            u8::from(e.b_over_capacity)
                                        )))
                                    }),
                            }
                        },
                    );
                    match slots {
                        [Err(err), _] | [_, Err(err)] => Err(err),
                        [Ok(p1), Ok(p2)] => {
                            let s1 = unsafe { xm(p1) };
                            let s2 = unsafe { xm(p2) };
                            if !s1.as_ref().is_some_and(|x| !x.item.is_empty())
                                && !s2.as_ref().is_some_and(|x| !x.item.is_empty())
                            {
                                Err(Cow::from("both slots are empty").into())
                            } else if p1 == p2 {
                                Err(Cow::from("slot1 and slot2 are the same").into())
                            } else {
                                let i1 = unsafe { s1.as_ref().unwrap().item.clone() };
                                let i2 = unsafe { s2.as_ref().unwrap().item.clone() };
                                let v1 = s1.as_ref().unwrap().vtable();
                                let v2 = s2.as_ref().unwrap().vtable();
                                let s1 = ptr::addr_of_mut!(*s1.unwrap());
                                let s2 = ptr::addr_of_mut!(*s2.unwrap());
                                if !i1.p_weapon.is_null() && unsafe { !v2.can_hold_weapon(s2) } {
                                    Err(Cow::from(
                                        "slot1 holds a weapon, but slot2 can't store weapons",
                                    )
                                    .into())
                                } else if !i2.p_weapon.is_null()
                                    && unsafe { !v1.can_hold_weapon(s1) }
                                {
                                    Err(Cow::from(
                                        "slot2 holds a weapon, but slot1 can't store weapons",
                                    )
                                    .into())
                                } else if !i1.p_drone.is_null() && unsafe { !v2.can_hold_drone(s2) }
                                {
                                    Err(Cow::from(
                                        "slot1 holds a drone, but slot2 can't store drones",
                                    )
                                    .into())
                                } else if !i2.p_drone.is_null() && unsafe { !v1.can_hold_drone(s1) }
                                {
                                    Err(Cow::from(
                                        "slot2 holds a drone, but slot1 can't store drones",
                                    )
                                    .into())
                                } else if !i1.p_crew.is_null() && unsafe { !v2.can_hold_crew(s2) } {
                                    Err(Cow::from(
                                        "slot1 holds a crew member, but slot2 can't store crew",
                                    )
                                    .into())
                                } else if !i2.p_crew.is_null() && unsafe { !v1.can_hold_crew(s1) } {
                                    Err(Cow::from(
                                        "slot2 holds a crew member, but slot1 can't store crew",
                                    )
                                    .into())
                                } else if !i1.augment.is_null()
                                    && unsafe { !v2.can_hold_augment(s2) }
                                {
                                    Err(Cow::from(
                                        "slot1 holds a augment, but slot2 can't store augments",
                                    )
                                    .into())
                                } else if !i2.augment.is_null()
                                    && unsafe { !v1.can_hold_augment(s1) }
                                {
                                    Err(Cow::from(
                                        "slot2 holds a augment, but slot1 can't store augments",
                                    )
                                    .into())
                                } else if i1.is_empty() {
                                    Err(Cow::from("slot1 holds no items").into())
                                } else if i2.is_empty() {
                                    Err(Cow::from("slot2 holds no items").into())
                                } else {
                                    unsafe {
                                        if !i1.is_empty() {
                                            v1.remove_item(s1);
                                        }
                                        if !i2.is_empty() {
                                            v2.remove_item(s2);
                                        }
                                        if !i2.is_empty() {
                                            v1.add_item(s1, i2);
                                        }
                                        if !i1.is_empty() {
                                            v2.add_item(s2, i1);
                                        }
                                    }
                                    Ok(Cow::from("successfully swapped the slots").into())
                                }
                            }
                        }
                    }
                }
            }
            FtlActions::Back(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't go back at the time").into())
                } else {
                    let gui = app.gui_mut().unwrap();
                    if gui.store_screens.base.b_open {
                        gui.store_screens.close();
                        Ok(Cow::from("closed the store").into())
                    } else if gui.star_map().unwrap().base.b_open {
                        let star_map = gui.star_map_mut().unwrap();
                        if star_map.b_choosing_new_sector {
                            star_map.b_choosing_new_sector = false;
                            Ok(Cow::from("closed next sector selection").into())
                        } else {
                            unsafe {
                                star_map
                                    .base
                                    .vtable()
                                    .close(ptr::addr_of_mut!(star_map.base));
                            }
                            Ok(Cow::from("closed the starmap").into())
                        }
                    } else if gui.ship_screens.base.b_open {
                        gui.ship_screens.close();
                        Ok(Cow::from("closed the ship overview").into())
                    } else {
                        Err(Cow::from("nothing to close").into())
                    }
                }
            }
            FtlActions::ShipOverview(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't open the ship overview at the time").into())
                } else {
                    let gui = app.gui_mut().unwrap();
                    if gui.danger_location {
                        Err(Cow::from(
                            "can't open the ship overview at the time because of the enemy ship",
                        )
                        .into())
                    } else if !gui.upgrade_screen.base.b_open {
                        gui.ship_screens.open();
                        Ok(Cow::from("successfully opened ship overview").into())
                    } else {
                        Err(
                            Cow::from("can't open the ship overview because it's already open")
                                .into(),
                        )
                    }
                }
            }
            FtlActions::UpgradeSystem(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't upgrade ship systems at the time").into())
                } else {
                    let system = event.system;
                    let gui = app.gui_mut().unwrap();
                    let upgrades = &mut gui.upgrade_screen;
                    if system.as_str() == "reactor" {
                        let cost = upgrades.reactor_button.reactor_cost();
                        let scrap = upgrades.ship_manager().unwrap().current_scrap;
                        if cost > scrap {
                            Err(Cow::from(format!(
                                "the reactor upgrade costs {cost} scrap, you only have {scrap}"
                            ))
                            .into())
                        } else if power_manager(upgrades.ship_manager().unwrap().i_ship_id)
                            .is_some_and(|x| {
                                x.current_power.second + upgrades.reactor_button.temp_upgrade >= 25
                            })
                        {
                            Err(Cow::from("the reactor is already at max power (25)").into())
                        } else {
                            let btn = &mut upgrades.reactor_button;
                            btn.base.base.b_hover = true;
                            unsafe {
                                btn.base
                                    .base
                                    .vtable()
                                    .on_click(ptr::addr_of_mut!(btn.base.base));
                            }
                            Ok(Cow::from("successfully updated the reactor").into())
                        }
                    } else {
                        let c = IdMap::with(|map| {
                            upgrades.v_upgrade_boxes.iter().copied().find(|x| {
                                unsafe { xc(*x).unwrap() }.system().is_some_and(|x| {
                                    System::from_id(x.i_system_type).is_some_and(|x| {
                                        map.map(x.blueprint().unwrap().title.to_str().into())
                                            == system
                                    })
                                })
                            })
                        });
                        if let Some(c) = c {
                            let b = unsafe { xc(c).unwrap() };
                            if b.system().unwrap().power_state.second + b.temp_upgrade
                                < b.system().unwrap().max_level
                            {
                                for b in upgrades.v_upgrade_boxes.iter() {
                                    let b = unsafe { xm(*b).unwrap() };
                                    if let Some(b) = b.current_button_mut() {
                                        b.base.b_hover = false;
                                    }
                                }
                                let b = unsafe { xm(c).unwrap() };
                                b.current_button_mut().unwrap().base.b_hover = true;
                                upgrades.base.b_close_button_selected = false;
                                upgrades.undo_button.base.b_hover = false;
                                upgrades.reactor_button.base.base.b_hover = false;
                                unsafe {
                                    upgrades.base.vtable().mouse_click(
                                        ptr::addr_of_mut!(upgrades.base),
                                        0,
                                        0,
                                    );
                                }
                                Ok(Cow::from(format!(
                                        "will upgrade the {} system to level {} once you leave the upgrades screen",
                                        b.blueprint().unwrap().desc.title.to_str(),
                                        b.system().unwrap().power_state.second + b.temp_upgrade),
                                    ).into())
                            } else {
                                Err(Cow::from(format!(
                                    "the system is already at max level ({})",
                                    b.system().unwrap().max_level
                                ))
                                .into())
                            }
                        } else {
                            let mut systems = Vec::new();
                            if gui.upgrade_screen.reactor_button.base.base.b_active {
                                systems.push(serde_json::Value::String("reactor".to_owned()));
                            }
                            IdMap::with(|map| {
                                for b in gui
                                    .upgrade_screen
                                    .v_upgrade_boxes
                                    .iter()
                                    .map(|x| unsafe { xc(*x).unwrap() })
                                {
                                    let Some(bp) = b.blueprint() else {
                                        continue;
                                    };
                                    systems.push(serde_json::Value::String(
                                        map.map(bp.name.to_str()).into_owned(),
                                    ));
                                }
                            });
                            Err(Cow::from(format!(
                                "this system can't be upgraded, upgradeable systems: {}",
                                serde_json::to_string(&systems).unwrap()
                            ))
                            .into())
                        }
                    }
                }
            }
            FtlActions::UndoUpgrades(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't undo the ship upgrades at the time").into())
                } else {
                    let gui = app.gui_mut().unwrap();
                    let upgrades = &mut gui.upgrade_screen;
                    upgrades.base.b_close_button_selected = false;
                    for b in upgrades.v_upgrade_boxes.iter() {
                        let b = unsafe { xm(*b).unwrap() };
                        if let Some(b) = b.current_button_mut() {
                            b.base.b_hover = false;
                        }
                    }
                    upgrades.undo_button.base.b_hover = true;
                    upgrades.reactor_button.base.base.b_hover = false;
                    unsafe {
                        upgrades
                            .base
                            .vtable()
                            .mouse_click(ptr::addr_of_mut!(upgrades.base), 0, 0);
                    }
                    Ok(Cow::from("ship upgrades undone").into())
                }
            }
            FtlActions::FireCrew(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't fire crew members at the time").into())
                } else {
                    let gui = app.gui_mut().unwrap();
                    let c = IdMap::with(|map| {
                        gui.ship_manager()
                            .unwrap()
                            .v_crew_list
                            .iter()
                            .find(|x| {
                                map.map(
                                    unsafe { xc(**x).unwrap() }
                                        .blueprint
                                        .crew_name_long
                                        .to_str(),
                                ) == event.name.as_str()
                            })
                            .copied()
                    });
                    if let Some(c) = c {
                        let crew = &mut gui.crew_screen;
                        if let Some(cc) = crew
                            .crew_boxes
                            .iter()
                            .map(|x| unsafe { xm(*x).unwrap() })
                            .find(|x| !x.base.item.is_empty() && x.base.item.p_crew == c)
                        {
                            if cc.b_show_delete {
                                cc.b_confirm_delete = true;
                                Ok(
                                    Cow::from("will fire the crew member after confirmation")
                                        .into(),
                                )
                            } else {
                                Err(Cow::from("can't delete the crew member").into())
                            }
                        } else {
                            Err(Cow::from(
                                "crew member button not found, this is probably a bug in the mod",
                            )
                            .into())
                        }
                    } else {
                        let names = IdMap::with(|map| {
                            app.gui()
                                .unwrap()
                                .ship_manager()
                                .unwrap()
                                .v_crew_list
                                .iter()
                                .map(|x| {
                                    map.map(
                                        unsafe { xc(*x).unwrap() }
                                            .blueprint
                                            .crew_name_long
                                            .to_str(),
                                    )
                                })
                                .collect::<Vec<_>>()
                        });
                        Err(Cow::from(format!(
                            "this crew member doesn't exist, current crew members: {}",
                            serde_json::to_string(&names).unwrap()
                        ))
                        .into())
                    }
                }
            }
            FtlActions::Jump(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't jump to a different star system at the time").into())
                } else {
                    let gui = app.gui_mut().unwrap();
                    let s = gui.star_map_mut().unwrap();
                    let loc = s.current_loc().unwrap();
                    let locs = loc.neighbors();
                    if let Some(path) = locs.get(&event.direction) {
                        s.potential_loc = *path;
                        s.ready_to_travel = true;
                        unsafe {
                            s.base.vtable().close(ptr::addr_of_mut!(s.base));
                        }
                        Ok(Cow::from("jumping...").into())
                    } else {
                        Err(Cow::from("there's no path in the direction you've chosen").into())
                    }
                }
            }
            FtlActions::StarMap(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't open the starmap").into())
                } else {
                    let gui = app.gui_mut().unwrap();
                    let ship = gui.ship_manager().unwrap();
                    if !ship
                        .system(System::Engines)
                        .is_some_and(|x| x.functioning())
                    {
                        Err(
                            Cow::from("Your Engine System must be functioning in order to Jump.")
                                .into(),
                        )
                    } else if !ship.system(System::Pilot).is_some_and(|x| x.functioning()) {
                        if ship.system(System::Pilot).is_some_and(|x| {
                            x.powered() && (!x.b_under_attack || x.i_hack_effect <= 1)
                        }) {
                            Err(Cow::from(
                                "You must have a crewmember in the Pilot System to Jump.",
                            )
                            .into())
                        } else {
                            Err(Cow::from(
                                "Your Pilot System must be functioning in order to Jump.",
                            )
                            .into())
                        }
                    } else if ship.jump_timer.first < ship.jump_timer.second {
                        Err(Cow::from("the ship's FTL drive hasn't yet charged").into())
                    } else {
                        let enemy = gui.enemy_ship();
                        let leaving_behind = enemy.is_some_and(|enemy| {
                            let enemy = enemy.ship_manager().unwrap();
                            enemy
                                .v_crew_list
                                .iter()
                                .map(|x| unsafe { xc(*x).unwrap() })
                                .filter(|x| unsafe {
                                    x.i_ship_id == 0
                                        && !x.b_dead
                                        && !x
                                            .vtable()
                                            .base
                                            .is_drone(ptr::addr_of!(**x).cast_mut().cast())
                                })
                                .count()
                                != 0
                        });
                        if leaving_behind {
                            let d = &mut gui.leave_crew_dialog;
                            unsafe {
                                d.base.vtable().open(ptr::addr_of_mut!(d.base));
                            }
                            Ok(Cow::from("will open the starmap after confirmation").into())
                        } else {
                            let s = gui.star_map_mut().unwrap();
                            unsafe {
                                s.base.vtable().open(ptr::addr_of_mut!(s.base));
                            }
                            Ok(Cow::from("opened the starmap").into())
                        }
                    }
                }
            }
            FtlActions::Wait(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't skip your turn at the time").into())
                } else {
                    let gui = app.gui_mut().unwrap();
                    let s = gui.star_map_mut().unwrap();
                    s.close_button.base.b_hover = false;
                    if s.distress_button.state != event.distress_signal {
                        s.distress_button.base.base.b_hover = true;
                        unsafe {
                            s.base.vtable().mouse_click(ptr::addr_of_mut!(s.base), 0, 0);
                        }
                    }
                    s.distress_button.base.base.b_hover = false;
                    s.wait_button.base.b_hover = true;
                    unsafe {
                        s.base.vtable().mouse_click(ptr::addr_of_mut!(s.base), 0, 0);
                    }
                    Ok(Cow::from("waiting...").into())
                }
            }
            FtlActions::NextSector(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't go to the next sector at the time").into())
                } else {
                    let gui = app.gui_mut().unwrap();
                    let s = gui.star_map_mut().unwrap();
                    if s.b_secret_sector {
                        s.close_button.base.b_hover = false;
                        s.distress_button.base.base.b_hover = false;
                        s.wait_button.base.b_hover = false;
                        s.end_button.base.b_hover = true;
                        Ok(Cow::from(
                            "you get moved to the bonus secret sector, can't select the next sector for now",
                        )
                        .into())
                    } else {
                        s.b_choosing_new_sector = true;
                        s.potential_sector_choice = -1;
                        Ok(Cow::from("opened next sector selection").into())
                    }
                }
            }
            FtlActions::ChooseNextSector(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't choose the next sector at the time").into())
                } else {
                    let gui = app.gui_mut().unwrap();
                    let s = gui.star_map_mut().unwrap();
                    let sec = s.current_sector().unwrap();
                    let secs = sec.neighbors();
                    if let Some(path) = secs.get(&event.direction) {
                        s.final_sector_choice = s
                            .sectors
                            .iter()
                            .enumerate()
                            .find(|(_, x)| **x == *path)
                            .unwrap()
                            .0 as i32;
                        Ok(Cow::from("jumping...").into())
                    } else {
                        Err(Cow::from("there's no path in the direction you've chosen").into())
                    }
                }
            }
            FtlActions::OpenStore(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't open the store at the time").into())
                } else {
                    let gui = app.gui_mut().unwrap();
                    gui.store_screens.open();
                    Ok(Cow::from("successfully opened the store").into())
                }
            }
            FtlActions::BuyScreen(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't switch to the buy screen at the time").into())
                } else {
                    let gui = app.gui_mut().unwrap();
                    let store = &mut gui.store_screens;
                    unsafe {
                        store.set_tab(0);
                    }
                    Ok(Cow::from("successfully opened the buy screen").into())
                }
            }
            FtlActions::SellScreen(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't switch to the sell screen at the time").into())
                } else {
                    let gui = app.gui_mut().unwrap();
                    let store = &mut gui.store_screens;
                    unsafe {
                        store.set_tab(1);
                    }
                    Ok(Cow::from("successfully opened the sell screen").into())
                }
            }
            FtlActions::SwitchStorePage(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't switch store pages at the time").into())
                } else {
                    let store = app
                        .world_mut()
                        .unwrap()
                        .base_location_event_mut()
                        .unwrap()
                        .store_mut()
                        .unwrap();
                    store.b_show_page2 = !store.b_show_page2;
                    store.current_button = if store.b_show_page2 {
                        ptr::addr_of_mut!(store.page2)
                    } else {
                        ptr::addr_of_mut!(store.page1)
                    };
                    Ok(Cow::from("successfully switched the store page").into())
                }
            }
            FtlActions::Sell(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't sell ites at the time, try opening the shop or switching to the sell tab").into())
                } else {
                    let gui = app.gui_mut().unwrap();
                    let e = &mut gui.equip_screen;
                    let actions::InventorySlot {
                        r#type: kind,
                        index,
                    } = event.slot;
                    if index == 0 {
                        return Err(Some(Cow::from("indices start at 1")));
                    }
                    let slot = match kind {
                        InventorySlotType::Cargo => {
                            let b = e.boxes::<bindings::EquipmentBox>();
                            let index = usize::from(index - 1);
                            b.get(index).copied().ok_or_else(|| {
                                Some(Cow::from(format!("there are only {} cargo slots", b.len())))
                            })
                        }
                        InventorySlotType::Weapon => {
                            let b = e.boxes::<bindings::WeaponEquipBox>();
                            let index = usize::from(index - 1);
                            b.get(index).copied().ok_or_else(|| {
                                Some(Cow::from(format!(
                                    "there are only {} weapon slots",
                                    b.len()
                                )))
                            })
                        }
                        InventorySlotType::Drone => {
                            let b = e.boxes::<bindings::DroneEquipBox>();
                            let index = usize::from(index - 1);
                            b.get(index).copied().ok_or_else(|| {
                                Some(Cow::from(format!("there are only {} drone slots", b.len())))
                            })
                        }
                        InventorySlotType::Augmentation => {
                            let b = e.boxes::<bindings::AugmentEquipBox>();
                            let index = usize::from(index - 1);
                            b.get(index).copied().ok_or_else(|| {
                                Some(Cow::from(format!(
                                    "there are only {} augment slots",
                                    b.len()
                                )))
                            })
                        }
                        InventorySlotType::AugmentationOverCapacity => e
                            .b_over_aug_capacity
                            .then_some(e.over_aug_box.cast())
                            .ok_or_else(|| {
                                Some(Cow::from(format!(
                                    "there are only {} over-capacity augment slots",
                                    u8::from(e.b_over_aug_capacity)
                                )))
                            }),
                        InventorySlotType::OverCapacity => e
                            .b_over_capacity
                            .then_some(e.overcapacity_box)
                            .ok_or_else(|| {
                                Some(Cow::from(format!(
                                    "there are only {} over-capacity slots",
                                    u8::from(e.b_over_capacity)
                                )))
                            }),
                    };
                    match slot {
                        Err(err) => Err(err),
                        Ok(slot) => {
                            e.b_selling_item = true;
                            e.b_dragging = true;
                            assert!(e.b_selling_item);
                            e.dragging_equip_box = e
                                .v_equipment_boxes
                                .iter()
                                .enumerate()
                                .find(|(_, x)| **x == slot)
                                .unwrap()
                                .0 as i32;
                            unsafe {
                                e.base.vtable().mouse_up(ptr::addr_of_mut!(e.base), 0, 0);
                            }
                            e.b_dragging = false;
                            e.b_selling_item = false;
                            Ok(Cow::from("successfully sold the item").into())
                        }
                    }
                }
            }
            FtlActions::BuyDrone(_)
            | FtlActions::BuySystem(_)
            | FtlActions::BuyWeapon(_)
            | FtlActions::BuyConsumable(_)
            | FtlActions::BuyAugmentation(_)
            | FtlActions::BuyCrew(_)
            | FtlActions::Repair1(_)
            | FtlActions::RepairAll(_) => {
                let (valid, error) = match &action {
                    FtlActions::BuyDrone(event) => (self.actions.valid(event), "drones"),
                    FtlActions::BuyWeapon(event) => (self.actions.valid(event), "weapons"),
                    FtlActions::BuyAugmentation(event) => {
                        (self.actions.valid(event), "augmentations")
                    }
                    FtlActions::BuyCrew(event) => (self.actions.valid(event), "crew"),
                    FtlActions::BuyConsumable(event) => (self.actions.valid(event), "items"),
                    FtlActions::BuySystem(event) => (self.actions.valid(event), "systems"),
                    FtlActions::Repair1(event) => (self.actions.valid(event), ""),
                    FtlActions::RepairAll(event) => (self.actions.valid(event), ""),
                    _ => unreachable!(),
                };
                if !valid {
                    if matches!(&action, FtlActions::Repair1(_) | FtlActions::RepairAll(_)) {
                        Err(Cow::from("can't repair your ship at the time").into())
                    } else {
                        Err(Cow::from(format!("can't buy {error} at the time")).into())
                    }
                } else {
                    let store = app
                        .world_mut()
                        .unwrap()
                        .base_location_event_mut()
                        .unwrap()
                        .store_mut()
                        .unwrap();
                    let b = match &action {
                        FtlActions::BuyDrone(event) => IdMap::with(|map| {
                            store
                                .active_boxes::<bindings::DroneStoreBox>()
                                .into_iter()
                                .find(|x| {
                                    unsafe { xc(*x).unwrap() }
                                        .blueprint()
                                        .map(|x| map.map(x.desc.title.to_str()))
                                        == Some(Cow::Borrowed(&event.drone_name))
                                })
                                .map(|x| ptr::addr_of_mut!(unsafe { xm(x).unwrap() }.base))
                        }),
                        FtlActions::BuyWeapon(event) => IdMap::with(|map| {
                            store
                                .active_boxes::<bindings::WeaponStoreBox>()
                                .into_iter()
                                .find(|x| {
                                    unsafe { xc(*x).unwrap() }
                                        .blueprint()
                                        .map(|x| map.map(x.desc.title.to_str()))
                                        == Some(Cow::Borrowed(&event.weapon_name))
                                })
                                .map(|x| ptr::addr_of_mut!(unsafe { xm(x).unwrap() }.base))
                        }),
                        FtlActions::BuyAugmentation(event) => IdMap::with(|map| {
                            store
                                .active_boxes::<bindings::AugmentStoreBox>()
                                .into_iter()
                                .find(|x| {
                                    unsafe { xc(*x).unwrap() }
                                        .blueprint()
                                        .map(|x| map.map(x.desc.title.to_str()))
                                        == Some(Cow::Borrowed(&event.augment_name))
                                })
                                .map(|x| ptr::addr_of_mut!(unsafe { xm(x).unwrap() }.base))
                        }),
                        FtlActions::BuyCrew(event) => IdMap::with(|map| {
                            store
                                .active_boxes::<bindings::CrewStoreBox>()
                                .into_iter()
                                .find(|x| {
                                    map.map(
                                        unsafe { xc(*x).unwrap() }.blueprint().desc.title.to_str(),
                                    ) == Cow::Borrowed(&event.crew_member_name)
                                })
                                .map(|x| ptr::addr_of_mut!(unsafe { xm(x).unwrap() }.base))
                        }),
                        FtlActions::BuyConsumable(event) => IdMap::with(|map| {
                            store
                                .active_boxes::<bindings::ItemStoreBox>()
                                .into_iter()
                                .find(|x| {
                                    unsafe { xc(*x).unwrap() }
                                        .blueprint()
                                        .map(|x| map.map(x.base.desc.title.to_str()))
                                        == Some(Cow::Borrowed(&event.item_name))
                                })
                                .map(|x| ptr::addr_of_mut!(unsafe { xm(x).unwrap() }.base))
                        }),
                        FtlActions::BuySystem(event) => IdMap::with(|map| {
                            store
                                .active_boxes::<bindings::SystemStoreBox>()
                                .into_iter()
                                .find(|x| {
                                    unsafe { xc(*x).unwrap() }
                                        .blueprint()
                                        .map(|x| map.map(x.desc.title.to_str()))
                                        == Some(Cow::Borrowed(&event.system_name))
                                })
                                .map(|x| ptr::addr_of_mut!(unsafe { xm(x).unwrap() }.base))
                        }),
                        FtlActions::Repair1(_) => store
                            .active_boxes::<bindings::RepairStoreBox>()
                            .into_iter()
                            .find(|x| !unsafe { xc(*x).unwrap() }.repair_all)
                            .map(|x| ptr::addr_of_mut!(unsafe { xm(x).unwrap() }.base)),
                        FtlActions::RepairAll(_) => store
                            .active_boxes::<bindings::RepairStoreBox>()
                            .into_iter()
                            .find(|x| unsafe { xc(*x).unwrap() }.repair_all)
                            .map(|x| ptr::addr_of_mut!(unsafe { xm(x).unwrap() }.base)),
                        _ => unreachable!(),
                    };
                    if let Some(c) = b {
                        let b = unsafe { xc(c).unwrap() };
                        if b.button.base.b_active {
                            store.base.b_close_button_selected = false;
                            store.current_button_mut().unwrap().base.b_hover = false;
                            for b in store.v_store_boxes.iter() {
                                let b = unsafe { xm(*b).unwrap() };
                                b.button.base.b_hover = false;
                            }
                            let b = unsafe { xm(c).unwrap() };
                            b.button.base.b_hover = true;
                            unsafe {
                                store.base.vtable().mouse_click(
                                    ptr::addr_of_mut!(store.base),
                                    0,
                                    0,
                                );
                            }
                            if matches!(&action, FtlActions::BuySystem(_))
                                && unsafe { xc(c.cast::<bindings::SystemStoreBox>()).unwrap() }
                                    .b_confirming
                            {
                                Ok(Cow::from(format!(
                                    "the purchase requires confirmations. Message: {}",
                                    unsafe { xc(c.cast::<bindings::SystemStoreBox>()).unwrap() }
                                        .confirm_string
                                        .to_str()
                                ))
                                .into())
                            } else if matches!(
                                action,
                                FtlActions::Repair1(_) | FtlActions::RepairAll(_)
                            ) {
                                let hull = store.shopper().unwrap().ship.hull_integrity;
                                Ok(Cow::from(format!(
                                    "successfully repaired ship hull to {}/{} HP",
                                    hull.first, hull.second
                                ))
                                .into())
                            } else {
                                Ok(Cow::from("successfully purchased the item").into())
                            }
                        } else {
                            Err(Cow::from("you don't have enough scrap for this purchase").into())
                        }
                    } else {
                        Err(Cow::from("the item you specified was not found").into())
                    }
                }
            }
            FtlActions::Pause(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't pause the game at the time").into())
                } else {
                    let gui = app.gui_mut().unwrap();
                    gui.b_paused = true;
                    Ok(Cow::from("successfully paused the game").into())
                }
            }
            FtlActions::Unpause(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't unpause the game at the time").into())
                } else {
                    let gui = app.gui_mut().unwrap();
                    gui.b_paused = false;
                    Ok(Cow::from("successfully unpaused the game").into())
                }
            }
            FtlActions::SystemsScreen(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't open the systems screen at the time").into())
                } else {
                    let gui = app.gui_mut().unwrap();
                    let overview = &mut gui.ship_screens;
                    unsafe {
                        overview.set_tab(0);
                    }
                    Ok(Cow::from("successfully opened the systems screen").into())
                }
            }
            FtlActions::CrewScreen(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't open the crew screen at the time").into())
                } else {
                    let gui = app.gui_mut().unwrap();
                    let overview = &mut gui.ship_screens;
                    unsafe {
                        overview.set_tab(1);
                    }
                    Ok(Cow::from("successfully opened the crew screen").into())
                }
            }
            FtlActions::InventoryScreen(event) => {
                if !self.actions.valid(&event) {
                    Err(Cow::from("can't open the inventory screen at the time").into())
                } else {
                    let gui = app.gui_mut().unwrap();
                    let overview = &mut gui.ship_screens;
                    unsafe {
                        overview.set_tab(2);
                    }
                    Ok(Cow::from("successfully opened the inventory screen").into())
                }
            }
        };
        if let Some(force) = &mut self.actions.force {
            if ret.is_ok() {
                self.actions.force = None;
            } else if force.send_at.is_none() {
                force.send_at = Some(Instant::now() + force.retry_timeout);
            }
        }
        ret
    }
    fn reregister_actions(&mut self) {
        if let Err(err) =
            self.register_actions_raw(self.actions.actions.values().cloned().collect())
        {
            log::error!("error reregistering actions: {err}");
        }
    }
}

fn is_zero<T: From<bool> + PartialEq>(x: &T) -> bool {
    *x == T::from(false)
}

#[derive(Serialize)]
struct ReactorState {
    power_used: i32,
    max_power: i32,
    #[serde(skip_serializing_if = "is_zero")]
    battery_power_used: i32,
    #[serde(skip_serializing_if = "is_zero")]
    max_battery_power: i32,
    #[serde(skip_serializing_if = "is_zero")]
    reduced_capacity: bool,
    #[serde(skip_serializing_if = "is_zero")]
    hacked: bool,
}

fn reactor_state(ship_id: i32) -> String {
    let Some(pow_man) = power_manager(ship_id) else {
        return "{}".to_owned();
    };
    let state = ReactorState {
        power_used: pow_man.current_power.first,
        max_power: (pow_man.current_power.second - pow_man.i_hacked - pow_man.i_temp_power_loss)
            .min(pow_man.i_temp_power_cap),
        battery_power_used: pow_man.battery_power.first,
        max_battery_power: pow_man.battery_power.second,
        reduced_capacity: pow_man.i_temp_power_loss != 0
            || pow_man.i_temp_power_cap < pow_man.current_power.second
            || pow_man.i_hacked != 0,
        hacked: pow_man.i_hacked > 0,
    };
    serde_json::to_string(&state).unwrap()
}

static mut GAME: OnceLock<State> = OnceLock::new();

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Force {
    query: Cow<'static, str>,
    context: Option<Cow<'static, str>>,
    ephemeral: bool,
    send_at: Option<Instant>,
    retry_timeout: Duration,
}

impl Force {
    pub fn new(query: impl Into<Cow<'static, str>>, timeout: Duration) -> Self {
        Self {
            query: query.into(),
            context: None,
            ephemeral: false,
            send_at: Some(Instant::now() + timeout),
            retry_timeout: timeout,
        }
    }
    /*pub fn with_context(mut self, ctx: impl Into<Cow<'static, str>>, ephemeral: bool) -> Self {
        self.context = Some(ctx.into());
        self.ephemeral = ephemeral;
        self
    }*/
}

#[derive(Default)]
struct ActionDb {
    actions: IndexMap<&'static str, neuro_sama::schema::Action>,
    force: Option<Force>,
    action_context: Option<(String, bool)>,
}

impl ActionDb {
    fn add<T: Action>(&mut self) {
        self.actions.insert(T::name(), meta::<T>());
    }
    fn valid<T: Action>(&self, _: &T) -> bool {
        self.actions.contains_key(&T::name())
    }
}

/*fn prop_opt(
    schema: &mut schemars::Schema,
    name: impl AsRef<str>,
) -> Option<&mut serde_json::Value> {
    schema
        .as_object_mut()?
        .get_mut("properties")?
        .get_mut(name.as_ref())
}*/

fn prop(schema: &mut schemars::Schema, name: impl AsRef<str>) -> &mut serde_json::Value {
    schema
        .as_object_mut()
        .unwrap()
        .get_mut("properties")
        .unwrap()
        .get_mut(name.as_ref())
        .unwrap()
}

fn array_item(schema: &mut serde_json::Value) -> &mut serde_json::Value {
    schema.as_object_mut().unwrap().get_mut("items").unwrap()
}

fn set_range<T: Copy + Into<serde_json::Number>>(
    schema: &mut serde_json::Value,
    range: std::ops::RangeInclusive<T>,
) {
    let min: serde_json::Number = (*range.start()).into();
    let max: serde_json::Number = (*range.end()).into();
    schema
        .as_object_mut()
        .unwrap()
        .insert("minimum".to_owned(), min.into());
    schema
        .as_object_mut()
        .unwrap()
        .insert("maximum".to_owned(), max.into());
}

fn set_enum(schema: &mut serde_json::Value, keep: impl FnMut(&mut serde_json::Value) -> bool) {
    schema
        .as_object_mut()
        .unwrap()
        .get_mut("enum")
        .unwrap()
        .as_array_mut()
        .unwrap()
        .retain_mut(keep);
}

fn add_enum(schema: &mut serde_json::Value, vals: Vec<serde_json::Value>) {
    schema
        .as_object_mut()
        .unwrap()
        .insert("enum".to_owned(), serde_json::Value::Array(vals));
}

fn iter_range<T: Copy + Ord>(it: impl Iterator<Item = T>) -> Option<std::ops::RangeInclusive<T>> {
    it.fold(None, |old, elem| {
        if let Some(old) = old {
            Some(std::cmp::min(*old.start(), elem)..=std::cmp::max(*old.end(), elem))
        } else {
            Some(elem..=elem)
        }
    })
}

fn available_actions(app: &CApp) -> ActionDb {
    let mut ret = ActionDb::default();
    if app.lang_chooser.base.b_open {
        // language selection is manual, while it's open don't allow neuro to do anything
        return ret;
    }
    if !app.game_logic {
        // if game logic isn't enabled, you can't do anything either
        return ret;
    }
    if app.menu.b_open {
        if app.menu.b_credit_screen {
            ret.add::<actions::SkipCredits>();
            return ret;
        }
        if app.menu.changelog.base.b_open {
            // changelog is manual
            return ret;
        }
        if app.menu.option_screen.base.base.b_open {
            // options are manual
            return ret;
        }
        if app.menu.ship_builder.b_open {
            let s = &app.menu.ship_builder;
            // TODO: (?) difficulty selection actions, enable advanced edition action
            ret.add::<actions::RenameCrew>();
            let mut meta = meta::<actions::RenameCrew>();
            let names = IdMap::with(|map| {
                s.v_crew_boxes
                    .iter()
                    .filter_map(|x| {
                        unsafe { xc(*x).unwrap() }.base.base.item.crew().map(|x| {
                            serde_json::Value::String(
                                map.map(x.blueprint.crew_name_long.to_str()).into_owned(),
                            )
                        })
                    })
                    .collect::<Vec<_>>()
            });
            add_enum(prop(&mut meta.schema, "oldName"), names);
            ret.actions.insert(actions::RenameCrew::name(), meta);
            ret.add::<actions::RenameShip>();
            ret.add::<actions::StartGame>();
            return ret;
        }
        if app.menu.b_score_screen {
            // scores are manual
            return ret;
        }
        if app.menu.b_select_save {
            if app.menu.confirm_new_game.base.b_open {
                if app.menu.confirm_new_game.yes_button.base.b_active {
                    ret.add::<actions::Confirm>();
                }
                if app.menu.confirm_new_game.no_button.base.b_active {
                    ret.add::<actions::Deny>();
                }
                ret.force = Some(Force::new(
                    app.menu.confirm_new_game.text.to_str().into_owned(),
                    Duration::from_secs(0),
                ));
                return ret;
            }
            return ret;
        }
        if app.menu.start_button.base.b_active {
            ret.add::<actions::NewGame>();
        }
        if app.menu.continue_button.base.b_active {
            ret.add::<actions::Continue>();
        }
        return ret;
    }
    // now, not main menu - command gui
    let gui = app.gui().unwrap();
    if gui.write_error_dialog.base.b_open {
        // idk what this is, require human intervention
        return ret;
    }
    if gui.leave_crew_dialog.base.b_open {
        if gui.leave_crew_dialog.yes_button.base.b_active {
            ret.add::<actions::Confirm>();
        }
        if gui.leave_crew_dialog.no_button.base.b_active {
            ret.add::<actions::Deny>();
        }
        ret.force = Some(Force::new(
            gui.leave_crew_dialog.text.to_str().into_owned(),
            Duration::from_secs(0),
        ));
        return ret;
    }
    if gui.game_over_screen.base.b_open {
        if gui.game_over_screen.b_showing_credits {
            ret.add::<actions::SkipCredits>();
        } else {
            ret.add::<actions::MainMenu>();
        }
        return ret;
    }
    if gui.menu_box.base.b_open {
        // pause menu, always manual *i think*
        return ret;
    }
    if gui.options_box.base.base.b_open {
        // options menu, always manual
        return ret;
    }
    if gui.star_map().unwrap().base.b_open {
        ret.add::<actions::Back>();
        let s = gui.star_map().unwrap();
        if s.current_loc.is_null() {
            return ret;
        }
        if s.b_choosing_new_sector {
            let sec = s.current_sector().unwrap();
            let secs: HashSet<_> = sec.neighbors().into_keys().map(|x| x.to_str()).collect();
            let mut meta = meta::<actions::ChooseNextSector>();
            set_enum(prop(&mut meta.schema, "direction"), |x| {
                secs.contains(x.as_str().unwrap())
            });
            ret.actions.insert(actions::ChooseNextSector::name(), meta);
        } else if s.wait_button.base.b_active {
            ret.add::<actions::Wait>();
        } else {
            let loc = s.current_loc().unwrap();
            let locs: HashSet<_> = loc.neighbors().into_keys().map(|x| x.to_str()).collect();
            let mut meta = meta::<actions::Jump>();
            set_enum(prop(&mut meta.schema, "direction"), |x| {
                locs.contains(x.as_str().unwrap())
            });
            ret.actions.insert(actions::Jump::name(), meta);
        }
        if s.end_button.base.b_active && !s.b_choosing_new_sector {
            ret.add::<actions::NextSector>();
        }
        return ret;
    }
    if gui.choice_box.base.b_open {
        let c = &gui.choice_box;
        for (i, choice) in c.choices.iter().enumerate() {
            let (name, mut meta) = match i {
                0 => (actions::Choose1::name(), meta::<actions::Choose1>()),
                1 => (actions::Choose2::name(), meta::<actions::Choose2>()),
                2 => (actions::Choose3::name(), meta::<actions::Choose3>()),
                3 => (actions::Choose4::name(), meta::<actions::Choose4>()),
                4 => (actions::Choose5::name(), meta::<actions::Choose5>()),
                5 => (actions::Choose6::name(), meta::<actions::Choose6>()),
                6 => (actions::Choose7::name(), meta::<actions::Choose7>()),
                7 => (actions::Choose8::name(), meta::<actions::Choose8>()),
                8 => (actions::Choose9::name(), meta::<actions::Choose9>()),
                _ => panic!(),
            };
            meta.description = format!(
                "Event option {}{}\n\n{}{}",
                i + 1,
                match choice.type_ {
                    1 => " (Requirements not met, cannot be chosen)",
                    2 => " (Requirements met)",
                    _ => " (No requirements)",
                },
                choice.text.to_str(),
                resource_event_str(&choice.rewards, gui.ship_manager().unwrap())
            )
            .into();
            ret.actions.insert(name, meta);
        }
        ret.action_context = Some((
            "Current event:\n".to_owned()
                + &c.main_text.to_str()
                + &resource_event_str(&c.rewards, gui.ship_manager().unwrap()),
            false,
        ));
        ret.force = Some(Force::new(
            ret.action_context.as_ref().unwrap().0.clone(),
            Duration::from_secs(10),
        ));
        return ret;
    }
    if gui.input_box.base.b_open {
        // this is for entering console commands i think? who cares ignore this
        return ret;
    }
    if gui.store_screens.base.b_open {
        let store = app
            .world()
            .unwrap()
            .base_location_event()
            .unwrap()
            .store()
            .unwrap();
        if gui.equip_screen.base.b_open {
            ret.add::<actions::SwapInventorySlots>();
            ret.add::<actions::Sell>();
            ret.add::<actions::BuyScreen>();
        }
        if store.base.b_open {
            if store.page2.base.b_active || store.page1.base.b_active {
                ret.add::<actions::SwitchStorePage>();
            }
            ret.add::<actions::SellScreen>();
            {
                let mut meta = meta::<actions::BuyAugmentation>();
                let boxes = store.active_boxes::<bindings::AugmentStoreBox>();
                if !boxes.is_empty() {
                    let augments: Vec<_> = IdMap::with(|map| {
                        boxes
                            .iter()
                            .filter_map(|x| {
                                unsafe { xc(*x).unwrap() }.blueprint().map(|x| {
                                    serde_json::Value::String(
                                        map.map(x.desc.title.to_str()).into_owned(),
                                    )
                                })
                            })
                            .collect()
                    });
                    add_enum(prop(&mut meta.schema, "augmentName"), augments);
                    ret.actions.insert(actions::BuyAugmentation::name(), meta);
                }
            }
            {
                let mut meta = meta::<actions::BuySystem>();
                let boxes = store.active_boxes::<bindings::SystemStoreBox>();
                if !boxes.is_empty() {
                    let systems: Vec<_> = IdMap::with(|map| {
                        boxes
                            .iter()
                            .filter_map(|x| {
                                unsafe { xc(*x).unwrap() }.blueprint().map(|x| {
                                    serde_json::Value::String(
                                        map.map(x.desc.title.to_str()).into_owned(),
                                    )
                                })
                            })
                            .collect()
                    });
                    add_enum(prop(&mut meta.schema, "systemName"), systems);
                    ret.actions.insert(actions::BuySystem::name(), meta);
                }
            }
            {
                let mut meta = meta::<actions::BuyWeapon>();
                let boxes = store.active_boxes::<bindings::WeaponStoreBox>();
                if !boxes.is_empty() {
                    let weapons: Vec<_> = IdMap::with(|map| {
                        boxes
                            .iter()
                            .filter_map(|x| {
                                unsafe { xc(*x).unwrap() }.blueprint().map(|x| {
                                    serde_json::Value::String(
                                        map.map(x.desc.title.to_str()).into_owned(),
                                    )
                                })
                            })
                            .collect()
                    });
                    add_enum(prop(&mut meta.schema, "weaponName"), weapons);
                    ret.actions.insert(actions::BuyWeapon::name(), meta);
                }
            }
            {
                let mut meta = meta::<actions::BuyDrone>();
                let boxes = store.active_boxes::<bindings::DroneStoreBox>();
                if !boxes.is_empty() {
                    let drones: Vec<_> = IdMap::with(|map| {
                        boxes
                            .iter()
                            .filter_map(|x| {
                                unsafe { xc(*x).unwrap() }.blueprint().map(|x| {
                                    serde_json::Value::String(
                                        map.map(x.desc.title.to_str()).into_owned(),
                                    )
                                })
                            })
                            .collect()
                    });
                    add_enum(prop(&mut meta.schema, "droneName"), drones);
                    ret.actions.insert(actions::BuyDrone::name(), meta);
                }
            }
            {
                let mut meta = meta::<actions::BuyCrew>();
                let boxes = store.active_boxes::<bindings::CrewStoreBox>();
                if !boxes.is_empty() {
                    let crew: Vec<_> = IdMap::with(|map| {
                        boxes
                            .iter()
                            .map(|x| {
                                serde_json::Value::String(
                                    map.map(
                                        unsafe { xc(*x).unwrap() }.blueprint().desc.title.to_str(),
                                    )
                                    .into_owned(),
                                )
                            })
                            .collect()
                    });
                    add_enum(prop(&mut meta.schema, "crewMemberName"), crew);
                    ret.actions.insert(actions::BuyCrew::name(), meta);
                }
            }
            {
                let mut meta = meta::<actions::BuyConsumable>();
                let boxes = store.active_boxes::<bindings::ItemStoreBox>();
                if !boxes.is_empty() {
                    let weapons: Vec<_> = IdMap::with(|map| {
                        boxes
                            .iter()
                            .filter_map(|x| {
                                unsafe { xc(*x).unwrap() }.blueprint().map(|x| {
                                    serde_json::Value::String(
                                        map.map(x.base.desc.title.to_str()).into_owned(),
                                    )
                                })
                            })
                            .collect()
                    });
                    add_enum(prop(&mut meta.schema, "itemName"), weapons);
                    ret.actions.insert(actions::BuyWeapon::name(), meta);
                }
            }
            let boxes = store.active_boxes::<bindings::RepairStoreBox>();
            let hull = gui.ship_manager().unwrap().ship.hull_integrity;
            if !boxes.is_empty() && hull.first < hull.second {
                ret.add::<actions::Repair1>();
                ret.add::<actions::RepairAll>();
            }
        }
        ret.add::<actions::Back>();
        return ret;
    }
    if gui.ship_screens.base.b_open {
        if gui.crew_screen.base.b_open {
            if gui.crew_screen.delete_dialog.base.b_open {
                if gui.crew_screen.delete_dialog.yes_button.base.b_active {
                    ret.add::<actions::Confirm>();
                }
                if gui.crew_screen.delete_dialog.no_button.base.b_active {
                    ret.add::<actions::Deny>();
                }
                ret.force = Some(Force::new(
                    gui.crew_screen.delete_dialog.text.to_str().into_owned(),
                    Duration::from_secs(0),
                ));
                return ret;
            }
            let names = IdMap::with(|map| {
                app.gui()
                    .unwrap()
                    .ship_manager()
                    .unwrap()
                    .v_crew_list
                    .iter()
                    .map(|x| {
                        serde_json::Value::String(
                            map.map(unsafe { xc(*x).unwrap() }.blueprint.crew_name_long.to_str())
                                .into_owned(),
                        )
                    })
                    .collect::<Vec<_>>()
            });
            let mut m = meta::<actions::RenameCrew>();
            add_enum(prop(&mut m.schema, "oldName"), names.clone());
            ret.actions.insert(actions::RenameCrew::name(), m);
            let mut m = meta::<actions::FireCrew>();
            add_enum(prop(&mut m.schema, "name"), names);
            ret.actions.insert(actions::FireCrew::name(), m);
        }
        if gui.equip_screen.base.b_open {
            ret.add::<actions::SwapInventorySlots>();
        }
        if gui.upgrade_screen.base.b_open {
            let mut systems = Vec::new();
            if gui.upgrade_screen.reactor_button.base.base.b_active {
                systems.push(serde_json::Value::String("reactor".to_owned()));
            }
            IdMap::with(|map| {
                for b in gui
                    .upgrade_screen
                    .v_upgrade_boxes
                    .iter()
                    .map(|x| unsafe { xc(*x).unwrap() })
                {
                    let Some(bp) = b.blueprint() else {
                        continue;
                    };
                    systems.push(serde_json::Value::String(
                        map.map(bp.name.to_str()).into_owned(),
                    ));
                }
            });
            let mut meta = meta::<actions::UpgradeSystem>();
            add_enum(prop(&mut meta.schema, "system"), systems);
            ret.actions.insert(actions::UpgradeSystem::name(), meta);
            if gui.upgrade_screen.undo_button.base.b_active {
                ret.add::<actions::UndoUpgrades>();
            }
        }
        if gui.ship_screens.current_tab != 0 {
            ret.add::<actions::SystemsScreen>();
        }
        if gui.ship_screens.current_tab != 1 {
            ret.add::<actions::CrewScreen>();
        }
        if gui.ship_screens.current_tab != 2 {
            ret.add::<actions::InventoryScreen>();
        }
        ret.add::<actions::Back>();
        return ret;
    }
    if gui.ship_manager().unwrap().b_jumping {
        // can't do anything if we're currently jumping and no popups are open, just wait
        return ret;
    }
    if gui.b_paused {
        ret.add::<actions::UnpauseGame>();
    } else {
        ret.add::<actions::PauseGame>();
    }
    if gui.ftl_button.base.base.b_active {
        ret.add::<actions::StarMap>();
    }
    // upgrade button (open ship_screens)
    if gui.upgrade_button.base.b_active {
        ret.add::<actions::ShipOverview>();
    }
    // store button (open store_screens)
    if gui.store_button.base.b_active {
        ret.add::<actions::OpenStore>();
    }
    // options button (open menu_box)
    // if gui.options_button.base.b_active {}
    // otherwise, no popups are open, so just do normal gameplay things i think idk
    // save crew positions button
    // if gui.crew_control.save_stations.base.b_active {}
    // load crew positions button
    // if gui.crew_control.return_stations.base.b_active {}
    let systems: HashMap<_, _> = IdMap::with(|map| {
        gui.ship_manager()
            .unwrap()
            .systems()
            .flat_map(|x| System::from_id(x.i_system_type))
            .map(|x| (map.map(x.blueprint().unwrap().title.to_str().into()), x))
            .collect()
    });

    // i can make it reregister the available systems per each action to only list the systems that
    // can currently be increased/decreased, but honestly whatever, i'd assume that reregistering
    // the actions too often is not a good idea but what do i know
    for (name, mut meta) in [
        (
            actions::IncreasePower::name(),
            meta::<actions::IncreasePower>(),
        ),
        (
            actions::DecreasePower::name(),
            meta::<actions::DecreasePower>(),
        ),
    ] {
        add_enum(
            prop(&mut meta.schema, "system"),
            systems
                .iter()
                .filter(|(_, v)| {
                    gui.sys_control
                        .ship_manager()
                        .unwrap()
                        .system(**v)
                        .is_some_and(|x| x.b_needs_power)
                })
                .map(|(k, _)| serde_json::Value::String(k.clone().into_owned()))
                .collect(),
        );
        ret.actions.insert(name, meta);
    }
    if gui.ship_manager().unwrap().weapon_system().is_some() {
        let cc = &gui.combat_control;
        let weapons: Vec<_> = IdMap::with(|map| {
            cc.weap_control
                .base
                .boxes
                .iter()
                .map(|x| x.cast::<bindings::WeaponBox>())
                .map(|x| unsafe { xc(x).unwrap() })
                .filter_map(|x| {
                    x.weapon().and_then(|x| {
                        x.blueprint().map(|x| {
                            serde_json::Value::String(map.map(x.desc.title.to_str()).into_owned())
                        })
                    })
                })
                .collect()
        });
        for (name, mut meta) in [
            (
                actions::ActivateWeapon::name(),
                meta::<actions::ActivateWeapon>(),
            ),
            (
                actions::DeactivateWeapon::name(),
                meta::<actions::DeactivateWeapon>(),
            ),
            (
                actions::SetWeaponTargets::name(),
                meta::<actions::SetWeaponTargets>(),
            ),
        ] {
            add_enum(prop(&mut meta.schema, "weaponName"), weapons.clone());
            /*if let Some(p) = prop_opt(&mut meta.schema, "targetRoomIds") {
                let Some(target) = gui.combat_control.current_target() else {
                    continue;
                };
                if let Some(range) = iter_range(
                    target
                        .ship_manager()
                        .unwrap()
                        .ship
                        .v_room_list
                        .iter()
                        .map(|room| unsafe { xc(*room).unwrap() }.i_room_id),
                ) {
                    set_range(array_item(p), range);
                }
            }*/
            ret.actions.insert(name, meta);
        }
    }
    if gui.ship_manager().unwrap().drone_system().is_some() {
        let cc = &gui.combat_control;
        let drones: Vec<_> = IdMap::with(|map| {
            cc.drone_control
                .base
                .boxes
                .iter()
                .map(|x| x.cast::<bindings::DroneBox>())
                .map(|x| unsafe { xc(x).unwrap() })
                .filter_map(|x| {
                    x.drone().and_then(|x| {
                        x.blueprint().map(|x| {
                            serde_json::Value::String(map.map(x.desc.title.to_str()).into_owned())
                        })
                    })
                })
                .collect()
        });
        for (name, mut meta) in [
            (
                actions::ActivateDrone::name(),
                meta::<actions::ActivateDrone>(),
            ),
            (
                actions::DeactivateDrone::name(),
                meta::<actions::DeactivateDrone>(),
            ),
        ] {
            add_enum(prop(&mut meta.schema, "droneName"), drones.clone());
            ret.actions.insert(name, meta);
        }
    }
    if let Some(sys) = gui.ship_manager().unwrap().hacking_system() {
        if !sys.b_hacking {
            if let Some(target) = gui.combat_control.current_target() {
                let mut meta = meta::<actions::HackSystem>();
                let systems = IdMap::with(|map| {
                    target
                        .ship_manager()
                        .unwrap()
                        .systems()
                        .map(|x| {
                            serde_json::Value::String(
                                map.map(
                                    System::from_id(x.i_system_type)
                                        .unwrap()
                                        .blueprint()
                                        .unwrap()
                                        .title
                                        .to_str()
                                        .into(),
                                )
                                .into_owned(),
                            )
                        })
                        .collect::<Vec<_>>()
                });
                add_enum(prop(&mut meta.schema, "system"), systems);
                ret.actions.insert(actions::HackSystem::name(), meta);
            }
        } else if sys.base.i_lock_count == 0 {
            ret.add::<actions::ActivateHacking>();
        }
    }
    if let Some(sys) = gui.ship_manager().unwrap().mind_system() {
        if sys.base.i_lock_count == 0 {
            ret.add::<actions::MindControl>();
        }
    }
    if let Some(sys) = gui.ship_manager().unwrap().battery_system() {
        if sys.base.i_lock_count == 0 && !sys.b_turned_on {
            ret.add::<actions::ActivateBattery>();
        }
    }
    if let Some(sys) = gui.ship_manager().unwrap().cloak_system() {
        if sys.base.i_lock_count == 0 && !sys.b_turned_on {
            ret.add::<actions::ActivateCloaking>();
        }
    }
    if let Some(sys) = gui.ship_manager().unwrap().teleport_system() {
        if sys.base.i_lock_count == 0 {
            if let Some(target) = gui.combat_control.current_target() {
                if let Some(range) = iter_range(
                    target
                        .ship_manager()
                        .unwrap()
                        .ship
                        .v_room_list
                        .iter()
                        .map(|room| unsafe { xc(*room).unwrap() }.i_room_id),
                ) {
                    let mut m = meta::<actions::TeleportSend>();
                    set_range(prop(&mut m.schema, "targetRoomId"), range.clone());
                    ret.actions.insert(actions::TeleportSend::name(), m);
                    let mut m = meta::<actions::TeleportReturn>();
                    set_range(prop(&mut m.schema, "sourceRoomId"), range);
                    ret.actions.insert(actions::TeleportReturn::name(), m);
                }
            }
        }
    }
    if let Some(sys) = gui.ship_manager().unwrap().system(System::Doors) {
        if sys.i_lock_count == 0 {
            if let Some(range) = iter_range(
                gui.ship_manager()
                    .unwrap()
                    .ship
                    .v_door_list
                    .iter()
                    .copied()
                    .map(|door| unsafe { xc(door).unwrap() }.i_door_id)
                    .chain(
                        gui.ship_manager()
                            .unwrap()
                            .ship
                            .v_outer_airlocks
                            .iter()
                            .copied()
                            .enumerate()
                            .map(|(i, _)| -(i as c_int + 1)),
                    ),
            ) {
                let mut m = meta::<actions::CloseDoors>();
                set_range(array_item(prop(&mut m.schema, "doorIds")), 0..=*range.end());
                ret.actions.insert(actions::CloseDoors::name(), m);
                let mut m = meta::<actions::OpenDoors>();
                set_range(array_item(prop(&mut m.schema, "doorIds")), range);
                ret.actions.insert(actions::OpenDoors::name(), m);
            }
            if let Some(range) = iter_range(
                gui.ship_manager()
                    .unwrap()
                    .ship
                    .v_room_list
                    .iter()
                    .map(|room| unsafe { xc(*room).unwrap() }.i_room_id),
            ) {
                let mut m = meta::<actions::PlanDoorRoute>();
                set_range(
                    array_item(prop(&mut m.schema, "firstRoomId")),
                    range.clone(),
                );
                set_range(array_item(prop(&mut m.schema, "secondRoomId")), range);
                ret.actions.insert(actions::PlanDoorRoute::name(), m);
            }
        }
    }
    let names = IdMap::with(|map| {
        app.gui()
            .unwrap()
            .ship_manager()
            .unwrap()
            .v_crew_list
            .iter()
            .filter(|x| unsafe { xc(**x).unwrap().vtable().get_controllable(**x) })
            .map(|x| {
                serde_json::Value::String(
                    map.map(unsafe { xc(*x).unwrap() }.blueprint.crew_name_long.to_str())
                        .into_owned(),
                )
            })
            .collect::<Vec<_>>()
    });
    let mut m = meta::<actions::MoveCrew>();
    add_enum(array_item(prop(&mut m.schema, "crewMemberNames")), names);
    ret.actions.insert(actions::MoveCrew::name(), m);
    let names1 = IdMap::with(|map| {
        app.gui()
            .unwrap()
            .ship_manager()
            .unwrap()
            .v_crew_list
            .iter()
            .filter(|x| unsafe { xc(**x).unwrap().vtable().has_special_power(**x) })
            .map(|x| {
                serde_json::Value::String(
                    map.map(unsafe { xc(*x).unwrap() }.blueprint.crew_name_long.to_str())
                        .into_owned(),
                )
            })
            .collect::<Vec<_>>()
    });
    let mut m = meta::<actions::Lockdown>();
    add_enum(array_item(prop(&mut m.schema, "crewMemberName")), names1);
    ret.actions.insert(actions::Lockdown::name(), m);
    // gui.sys_control.sys_boxes - iterate to get all the systems
    // 14 MindBox
    // 13 CloneBox
    // 15 HackBox
    // 9 TeleportBox
    // 10 CloakingBox
    // 11 ArtilleryBox
    // SystemBox
    // 3 WeaponSystemBox
    // 4 SystemBox
    ret
}

// power state: second is max, first is current, i think
// useful to hook: WarningMessage::Start or smth
// game_over.gameover_text, game_over.b_victory

pub fn loop_hook2(app: &mut CApp) {
    // activated with `l`, very useful for testing
    unsafe {
        (*super::SETTING_VALUES.0).command_console = true;
        GAME.get_or_init(|| {
            let (game2ws_tx, mut game2ws_rx) = mpsc::channel(128);
            let (ws2game_tx, ws2game_rx) = mpsc::channel(128);
            let state = State {
                tx: game2ws_tx,
                rx: ws2game_rx,
                app: ptr::null_mut(),
                actions: ActionDb::default(),
            };
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            std::thread::spawn(move || {
                rt.block_on(async {
                    loop {
                        if let Ok((mut ws, _)) = tokio_tungstenite::connect_async(
                            if let Ok(url) = std::env::var("NEURO_SDK_WS_URL") {
                                url
                            } else {
                                "ws://127.0.0.1:8000".to_owned()
                            },
                        )
                        .await
                        {
                            ws2game_tx.send(None).await.expect("ws->game channel closed");
                            loop {
                                tokio::select! {
                                    msg = game2ws_rx.recv() => {
                                        let Some(msg) = msg else {
                                            log::error!("game->ws channel closed");
                                            return;
                                        };
                                        log::info!("game2ws {msg:?}");
                                        if let Err(err) = ws.send(msg).await {
                                            log::error!("websocket send failed: {err}");
                                            break;
                                        }
                                    }
                                    msg = ws.next() => {
                                        let Some(msg) = msg else {
                                            break;
                                        };
                                        log::info!("ws2game {msg:?}");
                                        let msg = match msg {
                                            Ok(msg) => msg,
                                            Err(err) => {
                                                log::error!("receive error: {err}");
                                                continue;
                                            }
                                        };
                                        ws2game_tx.send(Some(msg)).await.expect("ws->game channel closed");
                                    }
                                }
                            }
                        }
                        log::info!("websocket connection closed, sleeping for 5 seconds");
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                });
            });
            state
        });
    }
    let game = unsafe { GAME.get_mut().unwrap() };
    game.app = app;
    let actions = available_actions(app);
    let mut to_remove = Vec::new();
    game.actions.actions.retain(|k, v| {
        if !matches!(actions.actions.get(*k), Some(x) if x == v) {
            to_remove.push((*k).into());
            false
        } else {
            true
        }
    });
    let mut changed = false;
    if !to_remove.is_empty() {
        changed = true;
        if let Err(err) = game.unregister_actions_raw(to_remove) {
            log::error!("error unregistering actions: {err}");
        }
    }
    let mut to_add = Vec::new();
    for (k, action) in actions.actions {
        if !game.actions.actions.contains_key(&k) {
            to_add.push(action.clone());
            game.actions.actions.insert(k, action);
        }
    }
    if !to_add.is_empty() {
        changed = true;
        if let Err(err) = game.register_actions_raw(to_add) {
            log::error!("error registering actions: {err}");
        }
    }
    if actions.force.is_none() {
        game.actions.force = None;
    } else if changed || game.actions.force != actions.force {
        game.actions.force = actions.force;
    }
    if let Some(ctx) = actions.action_context {
        if !game
            .actions
            .action_context
            .as_ref()
            .is_some_and(|x| x == &ctx)
        {
            if let Err(err) = game.context(ctx.0.clone(), ctx.1) {
                log::error!("error registering actions: {err}");
            }
            game.actions.action_context = Some(ctx);
        }
    } else {
        game.actions.action_context = None;
    }
    if let Some(mut force) = game.actions.force.clone() {
        if matches!(force.send_at, Some(x) if x < Instant::now()) {
            force.send_at = None;
            let mut builder = game
                .force_actions_raw(
                    force.query.clone(),
                    game.actions
                        .actions
                        .keys()
                        .copied()
                        .map(Cow::from)
                        .collect(),
                )
                .with_ephemeral_context(force.ephemeral);
            if let Some(ctx) = force.context.clone() {
                builder = builder.with_state(ctx.clone());
            }
            if let Err(err) = builder.send() {
                log::error!("error forcing actions: {err}");
            }
            game.actions.force = Some(force);
        }
    }
    while let Ok(msg) = game.rx.try_recv() {
        if let Some(msg) = msg {
            if let Err(err) = game.handle_message(msg) {
                log::error!("error handling message: {err}");
            }
        } else if let Err(err) = game.initialize() {
            log::error!("error starting up: {err}");
        }
    }
}

static DEACTIVATE: OnceLock<()> = OnceLock::new();

pub fn activated() -> bool {
    DEACTIVATE.get().is_none()
}

pub fn deactivate() {
    log::error!("deactivating");
    DEACTIVATE.get_or_init(|| {
        if let Some(game) = unsafe { GAME.get_mut() } {
            game.context("the mod just crashed... the game may or may not be still running, but you can no longer control it", false)
                .unwrap();
            game.tx = mpsc::channel(1).0;
        }
    });
}

pub unsafe fn loop_hook(app: *mut CApp) {
    if !activated() {
        return;
    }
    if !app.is_null() {
        #[allow(clippy::blocks_in_conditions)]
        if std::panic::catch_unwind(|| {
            loop_hook2(&mut *app);
        })
        .is_err()
        {
            deactivate();
        }
    }
}
