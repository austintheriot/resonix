use yew::{prelude::*, virtual_dom::AttrValue};

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
}

#[function_component(InputRange)]
pub fn input_range(props: &InputProps) -> Html {
    let disabled_class = if props.disabled { "disabled" } else { "" };

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
      </div>
    }
}
