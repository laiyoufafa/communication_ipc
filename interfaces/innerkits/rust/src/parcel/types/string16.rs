/*
 * Copyright (C) 2022 Huawei Device Co., Ltd.
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use super::*;
use crate::{ipc_binding, BorrowedMsgParcel, Result, result_status, AsRawPtr};
use std::ffi::{c_char, c_void};
use std::convert::TryInto;

pub struct String16(String);

impl String16 {
    pub fn new(value: &str) -> Self {
        Self(String::from(value))
    }
}

impl Serialize for String16 {
    fn serialize(&self, parcel: &mut BorrowedMsgParcel<'_>) -> Result<()> {
        let string = &self.0;
        // SAFETY: `parcel` always contains a valid pointer to a  `CParcel`
        let ret = unsafe {
            ipc_binding::CParcelWriteString16(
                parcel.as_mut_raw(), 
                string.as_ptr() as *const c_char,
                string.as_bytes().len().try_into().unwrap()  
            )};
        result_status::<()>(ret, ())
    }
}

impl Deserialize for String16 { 
    fn deserialize(parcel: &BorrowedMsgParcel<'_>) -> Result<Self> {
        let mut vec: Option<Vec<u8>> = None;
        let ok_status = unsafe {
            // SAFETY: `parcel` always contains a valid pointer to a  `CParcel`
            ipc_binding::CParcelReadString16(
                parcel.as_raw(), 
                &mut vec as *mut _ as *mut c_void,
                allocate_vec_with_buffer::<u8>
            )
        };
    
        if ok_status {
            let result = vec.map(|s| {
                println!("read string16 from native success, s: {:?}", s);
                match String::from_utf8(s) {
                    Ok(val) => val,
                    Err(_) => String::from("")
                }
            });
            if let Some(val) = result {
                Ok(Self(val))
            } else {
                println!("convert native string16 to String fail");
                Err(-1)
            }
        }else{
            println!("read string16 from native fail");
            Err(-1)
        }
    }
}
