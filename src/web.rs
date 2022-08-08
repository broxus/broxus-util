use wasm_bindgen::{JsCast, JsValue};

impl<T, E> HandleError for Result<T, E>
where
    E: ToString,
{
    type Output = T;

    fn handle_error(self) -> Result<Self::Output, JsValue> {
        self.map_err(|e| {
            let error = e.to_string();
            js_sys::Error::new(&error).unchecked_into()
        })
    }
}

pub trait HandleError {
    type Output;

    fn handle_error(self) -> Result<Self::Output, JsValue>;
}

pub struct ObjectBuilder {
    object: js_sys::Object,
}

impl ObjectBuilder {
    pub fn new() -> Self {
        Self {
            object: js_sys::Object::new(),
        }
    }

    pub fn set<T>(self, key: &str, value: T) -> Self
    where
        JsValue: From<T>,
    {
        let key = JsValue::from_str(key);
        let value = JsValue::from(value);
        js_sys::Reflect::set(&self.object, &key, &value).expect("shouldn't fail");
        self
    }

    pub fn build(self) -> JsValue {
        JsValue::from(self.object)
    }
}

impl Default for ObjectBuilder {
    fn default() -> Self {
        Self::new()
    }
}
