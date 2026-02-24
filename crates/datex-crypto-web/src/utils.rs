use wasm_bindgen::{JsError, JsValue};
use web_sys::{
    js_sys,
    js_sys::{Array, ArrayBuffer, Object, Reflect},
};

pub fn js_object<T: Into<JsValue>>(values: Vec<(&str, T)>) -> Object {
    let obj = Object::new();
    for (key, value) in values {
        let js_value: JsValue = value.into();
        let _ = Reflect::set(&obj, &key.into(), &js_value);
    }
    obj
}

pub fn js_array<T>(values: &[T]) -> JsValue
where
    T: Into<JsValue> + Clone,
{
    // FIXME TODO can we avoid clone here?
    let js_array = values
        .iter()
        .map(|x| <T as Into<JsValue>>::into(x.clone()))
        .collect::<Array>();

    JsValue::from(js_array)
}

pub trait TryAsByteSlice {
    fn try_as_u8_slice(&self) -> Result<Vec<u8>, JsError>;
}

pub trait AsByteSlice {
    fn as_u8_slice(&self) -> Vec<u8>;
}

impl TryAsByteSlice for JsValue {
    fn try_as_u8_slice(&self) -> Result<Vec<u8>, JsError> {
        let buffer: ArrayBuffer = self.clone().try_into().map_err(|_| {
            JsError::new("Failed to convert JsValue to ArrayBuffer")
        })?;

        Ok(buffer.as_u8_slice())
    }
}

impl AsByteSlice for ArrayBuffer {
    fn as_u8_slice(&self) -> Vec<u8> {
        let uint8_array = js_sys::Uint8Array::new(self);
        let mut bytes = vec![0; uint8_array.length() as usize];
        uint8_array.copy_to(&mut bytes);
        bytes
    }
}
