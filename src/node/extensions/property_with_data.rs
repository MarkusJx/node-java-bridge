use napi::{check_status, sys, Callback, Env, JsObject, NapiRaw, PropertyAttributes};
use std::ffi::CString;
use std::ptr;

pub(crate) struct PropertyWithData<T: 'static + Send + Sync + Clone> {
    name: CString,
    attributes: PropertyAttributes,
    getter: Option<Callback>,
    setter: Option<Callback>,
    data: T,
}

impl<T: 'static + Send + Sync + Clone> PropertyWithData<T> {
    pub fn new(name: String, data: T) -> napi::Result<Self> {
        Ok(Self {
            name: CString::new(name)?,
            attributes: PropertyAttributes::empty(),
            getter: None,
            setter: None,
            data,
        })
    }

    pub fn with_attributes(mut self, attributes: PropertyAttributes) -> Self {
        self.attributes = attributes;
        self
    }

    pub fn with_getter_and_setter(
        mut self,
        getter: Option<Callback>,
        setter: Option<Callback>,
    ) -> Self {
        self.getter = getter;
        self.setter = setter;
        self
    }

    fn raw(&self, object: &mut JsObject) -> napi::Result<sys::napi_property_descriptor> {
        let data = Box::into_raw(Box::new(self.data.clone()));

        object.add_finalizer(data, self.name.clone(), |ctx| unsafe {
            drop(Box::from_raw(ctx.value));
        })?;

        Ok(sys::napi_property_descriptor {
            utf8name: self.name.as_ptr(),
            name: ptr::null_mut(),
            method: None,
            getter: self.getter,
            setter: self.setter,
            value: ptr::null_mut(),
            attributes: self.attributes.into(),
            data: data as _,
        })
    }
}

pub(crate) trait DefinePropertiesWithData {
    fn define_properties_with_data<T: 'static + Send + Sync + Clone>(
        &mut self,
        env: &Env,
        properties: &Vec<PropertyWithData<T>>,
    ) -> napi::Result<()>;
}

impl DefinePropertiesWithData for JsObject {
    fn define_properties_with_data<T: 'static + Send + Sync + Clone>(
        &mut self,
        env: &Env,
        properties: &Vec<PropertyWithData<T>>,
    ) -> napi::Result<()> {
        check_status!(unsafe {
            sys::napi_define_properties(
                env.raw(),
                self.raw(),
                properties.len(),
                properties
                    .into_iter()
                    .map(|p| p.raw(self))
                    .collect::<napi::Result<Vec<_>>>()?
                    .as_ptr(),
            )
        })
    }
}
