use crate::database::NamePreference;
use gtk::prelude::*;
use gtk::{Align, Orientation};
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

pub struct NamePreferenceView {
	preference: Option<NamePreference>,
}

#[relm4::component(pub)]
impl SimpleComponent for NamePreferenceView {
	type Input = NamePreferenceInput;
	type Output = Option<NamePreference>;
	type Init = (&'static str, Option<NamePreference>);

	view! {
		gtk::Box {
			set_orientation: Orientation::Vertical,
			set_halign: Align::Start,

			gtk::Label {
				set_label: parent_name,
			},
			gtk::Box {
				set_orientation: Orientation::Horizontal,
				set_halign: Align::Center,

				#[name(favorite_button)]
				gtk::CheckButton {
					#[watch]
					set_active: model.preference == Some(NamePreference::Favorite),
					#[watch]
					set_inconsistent: model.preference.is_none(),
					connect_toggled[sender] => move |button| {
						if button.is_active() {
							sender.input(NamePreferenceInput::PreferenceToggled(NamePreference::Favorite));
						}
					}
				},
				gtk::Image {
					set_icon_name: Some("emblem-favorite-symbolic"),
				},
				#[name(nogo_button)]
				gtk::CheckButton {
					set_group: Some(&favorite_button),
					#[watch]
					set_active: model.preference == Some(NamePreference::NoGo),
					#[watch]
					set_inconsistent: model.preference.is_none(),
					connect_toggled[sender] => move |button| {
						if button.is_active() {
							sender.input(NamePreferenceInput::PreferenceToggled(NamePreference::NoGo));
						}
					}
				},
				gtk::Image {
					set_icon_name: Some("action-unavailable-symbolic"),
				},
			},
		}
	}

	fn init(
		(parent_name, preference): Self::Init,
		_root: Self::Root,
		sender: ComponentSender<Self>,
	) -> ComponentParts<Self> {
		let model = Self { preference };

		let widgets = view_output!();

		ComponentParts { widgets, model }
	}

	fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
		use NamePreferenceInput::*;
		match message {
			SetPreference(preference) => {
				self.preference = preference;
			}
			PreferenceToggled(preference) => {
				if Some(preference) == self.preference {
					return;
				}

				self.preference = Some(preference);
				let _ = sender.output(Some(preference));
			}
		}
	}
}

#[derive(Debug)]
pub enum NamePreferenceInput {
	SetPreference(Option<NamePreference>),
	PreferenceToggled(NamePreference),
}
