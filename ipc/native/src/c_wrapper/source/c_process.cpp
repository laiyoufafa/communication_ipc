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

#include "c_process.h"

#include <nativetoken_kit.h>
#include <token_setproc.h>
#include "c_remote_object_internal.h"
#include "ipc_skeleton.h"

using namespace OHOS;

CRemoteObject *GetContextManager(void)
{
    sptr<IRemoteObject> saMgr = IPCSkeleton::GetContextObject();
    if (saMgr == nullptr) {
        return nullptr;
    }
    CRemoteObject *holder = new (std::nothrow) CRemoteProxyHolder();
    if (holder == nullptr) {
        printf("%s: create samgr proxy holder failed\n", __func__);
        return nullptr;
    }
    holder->IncStrongRef(nullptr);
    holder->remote_ = saMgr;
    return holder;
}

void JoinWorkThread(void)
{
    IPCSkeleton::JoinWorkThread();
}

void InitTokenId(void)
{
    uint64_t tokenId;
    NativeTokenInfoParams infoInstance = {
        .dcapsNum = 0,
        .permsNum = 0,
        .aclsNum = 0,
        .dcaps = NULL,
        .perms = NULL,
        .acls = NULL,
        .processName = "com.ipc.test",
        .aplStr = "normal",
    };
    tokenId = GetAccessTokenId(&infoInstance);
    SetSelfTokenID(tokenId);
}