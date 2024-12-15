use gtk::{Align, Orientation, prelude::*};
use relm4::{ComponentParts, ComponentSender, SimpleComponent, gtk};

#[expect(clippy::struct_field_names)]
pub struct PreferenceFilterComponent {
	show_favorite_checkbox: gtk::CheckButton,
	show_nogo_checkbox: gtk::CheckButton,
	show_undecided_checkbox: gtk::CheckButton,
}

#[derive(Debug, Copy, Clone)]
pub struct PreferenceFilter {
	pub show_favorite: bool,
	pub show_nogo: bool,
	pub show_undecided: bool,
}

#[derive(Debug)]
pub enum PreferenceFilterInput {
	UpdateFilter,
}

#[derive(Debug)]
pub enum PreferenceFilterOutput {
	UpdateFilter(PreferenceFilter),
}

#[relm4::component(pub)]
impl SimpleComponent for PreferenceFilterComponent {
	type Input = PreferenceFilterInput;
	type Output = PreferenceFilterOutput;
	type Init = PreferenceFilter;

	view! {
		gtk::Box {
			set_orientation: Orientation::Horizontal,
			set_halign: Align::Center,
			set_spacing: 12,

			#[local]
			show_favorite_checkbox -> gtk::CheckButton {
				set_active: preference_filter.show_favorite,
				connect_toggled[sender] => move |_| {
					sender.input(PreferenceFilterInput::UpdateFilter);
				}
			},
			gtk::Image {
				set_icon_name: Some("emblem-favorite-symbolic"),
			},

			#[local]
			show_nogo_checkbox -> gtk::CheckButton {
				set_active: preference_filter.show_nogo,
				connect_toggled[sender] => move |_| {
					sender.input(PreferenceFilterInput::UpdateFilter);
				}
			},
			gtk::Image {
				set_icon_name: Some("action-unavailable-symbolic"),
			},

			#[local]
			show_undecided_checkbox -> gtk::CheckButton {
				set_label: Some("-"),
				set_active: preference_filter.show_undecided,
				connect_toggled[sender] => move |_| {
					sender.input(PreferenceFilterInput::UpdateFilter);
				}
			},
		}
	}

	fn init(preference_filter: Self::Init, root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
		let show_favorite_checkbox = gtk::CheckButton::new();
		let show_nogo_checkbox = gtk::CheckButton::new();
		let show_undecided_checkbox = gtk::CheckButton::new();
		let model = Self {
			show_favorite_checkbox: show_favorite_checkbox.clone(),
			show_nogo_checkbox: show_nogo_checkbox.clone(),
			show_undecided_checkbox: show_undecided_checkbox.clone(),
		};

		let widgets = view_output!();

		ComponentParts { model, widgets }
	}

	fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
		use PreferenceFilterInput::*;
		match message {
			UpdateFilter => {
				let _ = sender.output(PreferenceFilterOutput::UpdateFilter(PreferenceFilter {
					show_favorite: self.show_favorite_checkbox.is_active(),
					show_nogo: self.show_nogo_checkbox.is_active(),
					show_undecided: self.show_undecided_checkbox.is_active(),
				}));
			}
		}
	}
}
