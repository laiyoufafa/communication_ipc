/*
 * Copyright (C) 2021 Huawei Device Co., Ltd.
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

#include "ipc_workthread.h"
#include <pthread.h>
#include <sys/prctl.h>
#include "ipc_debug.h"
#include "ipc_process_skeleton.h"
#include "ipc_thread_skeleton.h"
#include "log_tags.h"

namespace OHOS {
#ifdef CONFIG_IPC_SINGLE
namespace IPC_SINGLE {
#endif

#ifndef TITLE
#define TITLE __PRETTY_FUNCTION__
#endif
#define DBINDER_LOGI(fmt, args...) \
    (void)OHOS::HiviewDFX::HiLog::Info(LABEL, "%{public}s %{public}d: " fmt, TITLE, __LINE__, ##args)

IPCWorkThread::IPCWorkThread(std::string threadName) : threadName_(std::move(threadName)) {}

IPCWorkThread::~IPCWorkThread()
{
    StopWorkThread();
}

void *IPCWorkThread::ThreadHandler(void *args)
{
    IPCWorkThread *threadObj = (IPCWorkThread *)args;
    IRemoteInvoker *invoker = IPCThreadSkeleton::GetRemoteInvoker(threadObj->proto_);
    DBINDER_LOGI("proto_=%{public}d,policy_=%{public}d", threadObj->proto_, threadObj->policy_);

    if (invoker != nullptr) {
        switch (threadObj->policy_) {
            case SPAWN_PASSIVE:
                invoker->JoinThread(false);
                break;
            case SPAWN_ACTIVE:
                invoker->JoinThread(true);
                break;
            case PROCESS_PASSIVE:
                invoker->JoinProcessThread(false);
                break;
            case PROCESS_ACTIVE:
                invoker->JoinProcessThread(true);
                break;
            default:
                DBINDER_LOGI("policy_ = %{public}d", threadObj->policy_);
                break;
        }
    }

    IPCProcessSkeleton *current = IPCProcessSkeleton::GetCurrent();
    if (current != nullptr) {
        current->OnThreadTerminated(threadObj->threadName_);
    }
    return nullptr;
}

void IPCWorkThread::StopWorkThread()
{
    IRemoteInvoker *invoker = IPCThreadSkeleton::GetRemoteInvoker(proto_);
    if (invoker != nullptr) {
        invoker->StopWorkThread();
    }
}

void IPCWorkThread::Start(int policy, int proto, std::string threadName)
{
    policy_ = policy;
    proto_ = proto;
    pthread_t threadId;
    int ret = pthread_create(&threadId, NULL, &IPCWorkThread::ThreadHandler, this);
    std::string wholeName = threadName + std::to_string(getpid()) + "_" + std::to_string(gettid());
    if (ret != 0) {
        DBINDER_LOGI("create thread failed");
    }
    DBINDER_LOGI("create thread = %{public}s, policy=%d, proto=%d", wholeName.c_str(), policy, proto);
    if (pthread_detach(threadId) != 0) {
        DBINDER_LOGI("detach error");
    }
}
#ifdef CONFIG_IPC_SINGLE
} // namespace IPC_SINGLE
#endif
} // namespace OHOS
