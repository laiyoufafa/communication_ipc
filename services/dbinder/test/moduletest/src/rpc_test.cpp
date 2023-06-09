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

#include "rpc_test.h"
#include "ipc_skeleton.h"
#include "ipc_types.h"

namespace OHOS {
std::string IRpcFooTest::GetFooName()
{
    return fooName_;
}

int RpcFooStub::OnRemoteRequest(uint32_t code,
    MessageParcel& data, MessageParcel& reply, MessageOption& option)
{
    int result = ERR_NONE;

    switch (code) {
        case GET_FOO_NAME: {
            reply.WriteString(TestGetFooName());
            break;
        }
        case GET_TOKENID: {
            result = TestAccessToken(data, reply);
            break;
        }
        default:
            return IPCObjectStub::OnRemoteRequest(code, data, reply, option);
    }

    return result;
}

std::string RpcFooStub::TestGetFooName(void)
{
    return GetFooName();
}

int32_t RpcFooStub::TestAccessToken(MessageParcel &data, MessageParcel &reply)
{
    uint32_t featureSet = 0;
    uint32_t tokenId = IPCSkeleton::GetCallingTokenID();
    if (tokenId != INVAL_TOKEN_ID) {
        featureSet = RPC_ACCESS_TOKEN_FLAG;
    }
    reply.WriteUint32(featureSet);
    reply.WriteUint32(tokenId);
    return ERR_NONE;
}

RpcFooProxy::RpcFooProxy(const sptr<IRemoteObject> &impl)
    : IRemoteProxy<IRpcFooTest>(impl)
{
}

std::string RpcFooProxy::TestGetFooName(void)
{
    MessageOption option;
    MessageParcel dataParcel, replyParcel;
    int err = Remote()->SendRequest(GET_FOO_NAME, dataParcel, replyParcel, option);
    if (err != 0) {
        return "";
    }
    return replyParcel.ReadString();
}

int32_t RpcFooProxy::TestAccessToken(MessageParcel &data, MessageParcel &reply)
{
    MessageOption option;
    int32_t err = Remote()->SendRequest(GET_TOKENID, data, reply, option);
    return err;
}
} // namespace OHOS