use adw::subclass::prelude::*;
use gtk::glib;
use gtk::CompositeTemplate;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/app/drey/paper-plane/ui/components/style_selection_row.ui")]
    pub(crate) struct StyleSelectionRow;

    #[glib::object_subclass]
    impl ObjectSubclass for StyleSelectionRow {
        const NAME: &'static str = "StyleSelectionRow";
        type Type = super::StyleSelectionRow;
        type ParentType = adw::PreferencesRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_css_name("styleselectionrow");
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for StyleSelectionRow {}
    impl WidgetImpl for StyleSelectionRow {}
    impl ListBoxRowImpl for StyleSelectionRow {}
    impl PreferencesRowImpl for StyleSelectionRow {}
}

glib::wrapper! {
    pub(crate) struct StyleSelectionRow(ObjectSubclass<imp::StyleSelectionRow>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Actionable;
}
