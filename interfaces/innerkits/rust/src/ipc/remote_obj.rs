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

use std::ptr;
use crate::{ipc_binding, IRemoteObj, Result, MsgParcel, BorrowedMsgParcel, AsRawPtr};
use crate::ipc_binding::{CRemoteObject, CDeathRecipient};
use crate::parcel::parcelable::{Serialize, Deserialize};
use std::ffi::{c_void};

/// RemoteObject can be used as proxy or stub object. 
/// It always contained a native CRemoteObject pointer.
/// # Invariant
///
/// `*mut CRemoteObject` must be valid
pub struct RemoteObj(ptr::NonNull<CRemoteObject>);

impl RemoteObj {
    /// Create an `RemoteObj` wrapper object from a raw `CRemoteObject` pointer.
    pub unsafe fn from_raw(obj: *mut CRemoteObject) -> Option<RemoteObj> {
        if obj.is_null() {
            None
        } else {
            Some(RemoteObj(unsafe{ptr::NonNull::new_unchecked(obj)}))
        }
    }

    /// Extract a raw `CRemoteObject` pointer from this wrapper.
    /// # Safety
    /// TODO
    pub unsafe fn as_inner(&self) -> *mut CRemoteObject {
        self.0.as_ptr()
    }
}

impl IRemoteObj for RemoteObj {
    fn send_request(&self, code: u32, data: &MsgParcel, is_async: bool) -> Result<MsgParcel> {
        // SAFETY: TODO
        unsafe {
            let mut reply = MsgParcel::new().expect("create reply MsgParcel not success");
            let result = ipc_binding::RemoteObjectSendRequest(self.as_inner(), code, data.as_raw(),
                reply.as_mut_raw(), is_async);
            if result == 0 {
                Ok(reply)
            } else {
                Err(result)
            }
        }
    }

    // Add death Recipient
    fn add_death_recipient(&self, recipient: &mut DeathRecipient) -> bool {
        unsafe {
            ipc_binding::AddDeathRecipient(self.as_inner(), recipient.as_mut_raw())
        }
    }

    // remove death Recipients
    fn remove_death_recipient(&self, recipient: &mut DeathRecipient) -> bool {
        unsafe {
            ipc_binding::RemoveDeathRecipient(self.as_inner(), recipient.as_mut_raw())
        }
    }
}

impl Serialize for RemoteObj {
    fn serialize(&self, parcel: &mut BorrowedMsgParcel<'_>) -> Result<()> {
        let ret = unsafe {
            ipc_binding::CParcelWriteRemoteObject(parcel.as_mut_raw(), self.as_inner())
        };
        if ret == true {
            Ok(())
        } else {
            Err(-1)
        }
    }
}

impl Deserialize for RemoteObj {
    fn deserialize(parcel: &BorrowedMsgParcel<'_>) -> Result<Self> {
        // Safety: `Parcel` always contains a valid pointer to an
        // `AParcel`. We pass a valid, mutable pointer to `val`, a
        // literal of type `$ty`, and `$read_fn` will write the
        let object = unsafe {
            let remote = ipc_binding::CParcelReadRemoteObject(parcel.as_raw());
            Self::from_raw(remote)
        };
        if let Some(x) = object {
            Ok(x)
        } else {
            Err(-1)
        }
    }
}

/// # Safety
///
/// An `RemoteObj` is an immutable handle to CRemoteObject, which is thread-safe
unsafe impl Send for RemoteObj {}
/// # Safety
///
/// An `RemoteObj` is an immutable handle to CRemoteObject, which is thread-safe
unsafe impl Sync for RemoteObj {}

impl Clone for RemoteObj {
    fn clone(&self) -> Self {
        // SAFETY: TODO 
        unsafe {
            ipc_binding::RemoteObjectIncStrongRef(self.as_inner());
        }
        // SAFETY: no `None` here, cause `self` is valid 
        Self(self.0)
    }
}

impl Drop for RemoteObj {
    fn drop(&mut self) {
        // SAFETY: TODO
        unsafe {
            ipc_binding::RemoteObjectDecStrongRef(self.as_inner());
        }
    }
}

#[repr(C)]
pub struct DeathRecipient {
    native: *mut CDeathRecipient,
    callback: *mut c_void,
}

impl DeathRecipient {
    pub fn new<F>(callback: F) -> Option<DeathRecipient>
    where
        F: Fn() + Send + Sync + 'static,
    {
        let callback = Box::into_raw(Box::new(callback));
        let native = unsafe {
            // set callback pointer to native, so we can find call which fuction
            // when remote service died.
            ipc_binding::CreateDeathRecipient(Self::on_remote_died::<F>,
                Self::on_destroy::<F>, callback as *mut c_void)
        };
        if native.is_null() {
            None
        } else {
            Some(DeathRecipient {
                native,
                callback: callback as *mut c_void,
            })
        }
    }

    /// Callback when remote service died by native.
    ///
    /// # Safety
    ///
    /// The callback parameter will be kept valid during native
    /// CDeathRecipient object lifetime.
    unsafe extern "C" fn on_remote_died<F>(callback: *mut c_void)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let callback = (callback as *const F).as_ref().unwrap();
        callback();
    }

    /// Callback when native CDeathRecipient destroyed
    ///
    /// # Safety
    ///
    /// The callback parameter will be kept valid during native
    /// CDeathRecipient object lifetime.
    unsafe extern "C" fn on_destroy<F>(callback: *mut c_void)
    where
        F: Fn() + Send + Sync + 'static,
    {
        if !callback.is_null() {
            println!("death recipient on destroy");
            Box::from_raw(callback as *mut F);
        }
    }
}

/// # Safety
///
/// A `DeathRecipient` is always constructed with a valid raw pointer
/// to a `CDeathRecipient`.
unsafe impl AsRawPtr<CDeathRecipient> for DeathRecipient {
    fn as_raw(&self) -> *const CDeathRecipient {
        self.native
    }

    fn as_mut_raw(&mut self) -> *mut CDeathRecipient {
        self.native
    }
}

impl Drop for DeathRecipient {
    fn drop(&mut self) {
        unsafe {
            // Safety: DeathRecipient will always hold a reference for
            // native CDeathRecipient.
            ipc_binding::DeathRecipientDecStrongRef(self.native);
        }
    }
}