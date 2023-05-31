use std::ops::Deref;
use std::rc::Rc;

use uuid::Uuid;
use yew::{prelude::*, virtual_dom::AttrValue};

pub struct GetLabelCallback {
    callback: Rc<dyn Fn(f64) -> Option<String>>,
    uuid: Uuid,
}

impl GetLabelCallback {
    pub fn new<F: Fn(f64) -> Option<String> + 'static>(cb: Rc<F>) -> Self {
        Self {
            callback: Rc::clone(&cb) as Rc<dyn Fn(f64) -> Option<String>>,
            uuid: Uuid::new_v4(),
        }
    }
}

impl PartialEq for GetLabelCallback {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl<F: Fn(f64) -> Option<String> + 'static> From<F> for GetLabelCallback {
    fn from(cb: F) -> Self {
        Self {
            callback: Rc::new(cb) as Rc<dyn Fn(f64) -> Option<String>>,
            uuid: Uuid::new_v4(),
        }
    }
}

impl<F: Fn(f64) -> Option<String> + 'static> From<Rc<F>> for GetLabelCallback {
    fn from(cb: Rc<F>) -> Self {
        Self {
            callback: cb as Rc<dyn Fn(f64) -> Option<String>>,
            uuid: Uuid::new_v4(),
        }
    }
}

impl Deref for GetLabelCallback {
    type Target = Rc<dyn Fn(f64) -> Option<String>>;
    fn deref(&self) -> &Self::Target {
        &self.callback
    }
}

impl Default for GetLabelCallback {
    fn default() -> Self {
        Self {
            callback: Rc::new(|_| None),
            uuid: Uuid::new_v4(),
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct InputProps {
    #[prop_or_default]
    pub disabled: bool,
    pub label: AttrValue,
    pub id: AttrValue,
    pub value: AttrValue,
    #[prop_or("0.0".into())]
    pub min: AttrValue,
    #[prop_or("1.0".into())]
    pub max: AttrValue,
    #[prop_or("0.001".into())]
    pub step: AttrValue,
    #[prop_or_default]
    pub oninput: Callback<InputEvent>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub onblur: Callback<FocusEvent>,
    #[prop_or_default]
    pub onfocus: Callback<FocusEvent>,
    #[prop_or_default]
    pub get_label_on_input: GetLabelCallback,
}

#[function_component(InputRange)]
pub fn input_range(props: &InputProps) -> Html {
    let disabled_class = if props.disabled { "disabled" } else { "" };
    let _value_label: UseStateHandle<Option<String>> =
        use_state_eq(|| (props.get_label_on_input)(props.value.clone().parse::<f64>().unwrap()));

    let value = props.value.clone();
    let value_label = (props.get_label_on_input)(value.parse::<f64>().unwrap());

    html! {
      <div class={classes!("input-range", disabled_class, props.class.clone())}>
        <label for={props.id.clone()}>{&props.label}</label>
        <div class="input-range-input-container">
            <input
                id={props.id.clone()}
                type="range"
                min={props.min.clone()}
                max={props.max.clone()}
                step={props.step.clone()}
                value={props.value.clone()}
                oninput={&props.oninput}
                onfocus={&props.onfocus}
                onblur={&props.onblur}
                disabled={props.disabled}
            />
        </div>
        {if let Some(value_label) = value_label {
            html!{
              <span
                class="input-range-value-label"
              >
                {value_label}
              </span>
            }
        } else {
            html!{}
        }}
      </div>
    }
}
