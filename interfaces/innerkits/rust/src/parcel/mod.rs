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

pub mod parcelable;
pub mod types;

use crate::{ipc_binding, Result};
use crate::ipc_binding::{CParcel};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::{Drop};
use std::ptr::{NonNull};
use crate::AsRawPtr;
use crate::parcel::parcelable::{Serialize, Deserialize};

/// Container for a message (data and object references) that can be sent
/// through Binder.
///
/// This type represents a parcel that is owned by Rust code.
#[repr(transparent)]
pub struct MsgParcel {
    ptr: NonNull<CParcel>,
}

unsafe impl Send for MsgParcel {}

impl MsgParcel {
    pub fn new() -> Option<Self> {
        let cparcel: *mut CParcel = unsafe {
            ipc_binding::CParcelObtain()
        };

        NonNull::new(cparcel).map(|x| MsgParcel{ptr: x})
    }

    pub unsafe fn from_raw(ptr: *mut CParcel) -> Option<MsgParcel> {
        NonNull::new(ptr).map(|ptr| Self { ptr })
    }

    pub fn into_raw(self) -> *mut CParcel {
        let ptr = self.ptr.as_ptr();
        let _ = ManuallyDrop::new(self);
        ptr
    }

    /// Get a borrowed view into the contents of this `MsgParcel`.
    pub fn borrowed(&mut self) -> BorrowedMsgParcel<'_> {
        // SAFETY: The raw pointer is a valid pointer 
        BorrowedMsgParcel {
            ptr: self.ptr,
            _mark: PhantomData,
        }
    }
}

/// # Safety
///
/// The `MsgParcel` constructors guarantee that a `MsgParcel` object will always
/// contain a valid pointer to an `CParcel`.
unsafe impl AsRawPtr<CParcel> for MsgParcel {
    fn as_raw(&self) -> *const CParcel {
        self.ptr.as_ptr()
    }

    fn as_mut_raw(&mut self) -> *mut CParcel {
        self.ptr.as_ptr()
    }
}

impl Drop for MsgParcel {
    fn drop(&mut self) {
        unsafe {
            ipc_binding::CParcelDecStrongRef(self.as_mut_raw())
        }  
    }
}

/// Container for a message (data and object references) that can be sent
/// through Binder.
///
/// This object is a borrowed variant of [`MsgParcel`]
#[repr(transparent)]
pub struct BorrowedMsgParcel<'a> {
    ptr: NonNull<CParcel>,
    _mark: PhantomData<&'a mut MsgParcel>,
}

impl<'a> BorrowedMsgParcel<'a> {

    /// # Safety:
    ///
    /// `*mut CParcel` must be a valid pointer 
    pub unsafe fn from_raw(ptr: *mut CParcel) -> Option<BorrowedMsgParcel<'a>> {
        Some(Self {
            ptr: NonNull::new(ptr)?,
            _mark: PhantomData,
        })
    }

    /// Get a sub-reference to this reference to the parcel.
    pub fn reborrow(&mut self) -> BorrowedMsgParcel<'_> {
        BorrowedMsgParcel {
            ptr: self.ptr,
            _mark: PhantomData,
        }
    }
}

/// # Safety
///
/// The `BorrowedMsgParcel` constructors guarantee that a `BorrowedMsgParcel` object
/// will always contain a valid pointer to an `CParcel`.
unsafe impl<'a> AsRawPtr<CParcel> for BorrowedMsgParcel<'a> {
    fn as_raw(&self) -> *const CParcel {
        self.ptr.as_ptr()
    }

    fn as_mut_raw(&mut self) -> *mut CParcel {
        self.ptr.as_ptr()
    }
}


impl MsgParcel {
    pub fn read<D: Deserialize>(&self) -> Result<D> {
        self.borrowed_ref().read()
    }

    pub fn write<S: Serialize + ?Sized>(&mut self, parcelable: &S) -> Result<()> {
        self.borrowed().write(parcelable)
    }

    /// Get an immutable borrowed view into the contents of this `MsgParcel`.
    pub fn borrowed_ref(&self) -> &BorrowedMsgParcel<'_> {
        // Safety: MsgParcel and BorrowedParcel are both represented in the same
        // way as a NonNull<CParcel> due to their use of repr(transparent),
        // so casting references as done here is valid.
        unsafe {
            &*(self as *const MsgParcel as *const BorrowedMsgParcel<'_>)
        }
    }
}

impl<'a> BorrowedMsgParcel<'a> {
    pub fn read<D: Deserialize>(&self) -> Result<D> {
        D::deserialize(self)
    }

    pub fn write<S: Serialize + ?Sized>(&mut self, parcelable: &S) -> Result<()> {
        parcelable.serialize(self)
    }
}