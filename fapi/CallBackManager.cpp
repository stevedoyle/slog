/**
 * @file CallBackManager.cpp
 *
 * @date 23 March 2005
 *
 * @brief The CallBackManager stores client callback references.
 * 
 * The CallBackManager is a singleton. It manages the storage and retrieval of 
 * client callback references. It uses a unique key, which is returned to the 
 * client, to store the callback references. The references store data relating
 * to the success of an operation and to help a client to identify an exact 
 * instance of a registered callback.
 *
 *
 * -- Intel Copyright Notice --
 *
 * @par
 * INTEL CONFIDENTIAL
 *
 * @par
 * Copyright 2005 Intel Corporation All Rights Reserved
 *
 * @par
 * The source code contained or described herein and all documents
 * related to the source code ("Material") are owned by Intel Corporation
 * or its suppliers or licensors.  Title to the Material remains with
 * Intel Corporation or its suppliers and licensors.  The Material
 * contains trade secrets and proprietary and confidential information of
 * Intel or its suppliers and licensors.  The Material is protected by
 * worldwide copyright and trade secret laws and treaty provisions. No
 * part of the Material may be used, copied, reproduced, modified,
 * published, uploaded, posted, transmitted, distributed, or disclosed in
 * any way without Intel's prior express written permission.
 *
 * @par
 * No license under any patent, copyright, trade secret or other
 * intellectual property right is granted to or conferred upon you by
 * disclosure or delivery of the Materials, either expressly, by
 * implication, inducement, estoppel or otherwise.  Any license under
 * such intellectual property rights must be express and approved by
 * Intel in writing.
 *
 * @par
 * For further details, please see the file README.TXT distributed with
 * this software.
 * -- End Intel Copyright Notice –
 */

/*
 * User defined include files required.
 */
#include "CallBackManager.h"
#include "APISimConfig.h"
#include "TraceMacro.h"
#include "FAPIDefs.h"

CallBackManager::CallBackManager()
: m_handleID(1)
{
}

CallBackManager::~CallBackManager()
{

}

CallBackManager& CallBackManager::instance()
{
    // Singleton Pattern
    static CallBackManager instance;
    return instance;
}



/**
 * Function Definition: registerCallBack(NPF_userContext_t userContext, 
 *                                       NPF_F_ATM_ConfigMgr_CallBackFunc_t callbackFunc,
 *                                       NPF_callbackHandle_t *callbackHandle) 
 */
NPF_error_t CallBackManager::RegisterCallBack(NPF_userContext_t userContext, 
                                       NPF_F_ATM_ConfigMgr_CallBackFunc_t callbackFunc,
                                       NPF_callbackHandle_t *callbackHandle)
{   
    APISimTrace(3,"Trace Level 3: CallBackManager::RegisterCallBack(%d,..,%d)\n",userContext, *callbackHandle); 
    tableIterator tableIter;
    
    // Check for duplicate registration of userContext and callbackFuntion
    // If found return stored handle and error message entry exists.
    pthread_mutex_lock(&syncMutex);
   
    for(tableIter = m_callbackTable.begin(); tableIter != m_callbackTable.end(); ++tableIter)
    {
        if(tableIter->second.m_context == userContext)
        {
            if(tableIter->second.m_function == callbackFunc)
            {
                APISimTrace(1,"Trace Level 1: CallBackManager::RegisterCallBack - Callback already registered!\n");
                *callbackHandle = tableIter->first;
                return NPF_E_RESOURCE_EXISTS;
            }    
        }
    }
        
    pthread_mutex_unlock(&syncMutex);

    // Create a callback entry object to be stored in the map.
    // The callback object holds the userContext and callbackfunc.
    CallBack entry = CallBack(userContext, callbackFunc);    
    
    // Insert entry into the map, with the current handleID value as key.
    pthread_mutex_lock(&syncMutex);
    
    // Check to see we do not exceed max number of callbacks
    if(m_callbackTable.size() == _ATM_FAPI_SIM_CB_HANDLE_MAX)
    {
        APISimTrace(1,"Trace Level 1: CallBackManager::RegisterCallBack - Max Number of Callbacks Reached!\n");
        return NPF_E_UNKNOWN;      
    }
    
    m_callbackTable.insert(callbackEntry(m_handleID, entry));
    pthread_mutex_unlock(&syncMutex);
    
    // Return callbackHandle of stored entry.
    *callbackHandle = m_handleID;
    
    // Increment callbackHandle value.
    m_handleID++;    
    // Return success.
    return NPF_NO_ERROR;
}

NPF_error_t CallBackManager::DeRegisterCallBack(NPF_callbackHandle_t cbHandle)
{
    APISimTrace(3,"Trace Level 3: CallBackManager::DeRegisterCallBack(%d)\n",cbHandle);
    if(cbHandle > _ATM_FAPI_SIM_CB_HANDLE_MAX)
    {
        APISimTrace(1,"Trace Level 1: CallBackManager::DeRegisterCallBack - Invalid CallBack Handle!\n");
        return NPF_E_BAD_CALLBACK_HANDLE;    
    } 
    
    pthread_mutex_lock(&syncMutex);
    tableFindIterator callbackIter = m_callbackTable.find(cbHandle);
    pthread_mutex_unlock(&syncMutex);
    
    if(callbackIter == m_callbackTable.end())
    {
        APISimTrace(1,"Trace Level 1: CallBackManager::DeRegisterCallBack - Callback Does Not Exist!\n");
        return NPF_E_BAD_CALLBACK_HANDLE; 
    }
    
    pthread_mutex_lock(&syncMutex);
    m_callbackTable.erase(cbHandle);
    pthread_mutex_unlock(&syncMutex);
    
    m_handleID--;
    return NPF_NO_ERROR;
    
       
}
/**
 * Function Definition: retrieveCallback(NPF_callbackHandle_t cbHandle)
 */
CallBack* CallBackManager::RetrieveCallback(NPF_callbackHandle_t cbHandle)
{
    APISimTrace(3,"Trace Level 3: CallBackManager::RetrieveCallback(%d)\n",cbHandle);
    pthread_mutex_lock(&syncMutex);
    tableFindIterator callbackIter = m_callbackTable.find(cbHandle);
    pthread_mutex_unlock(&syncMutex);
    if(callbackIter == m_callbackTable.end())
    {
        APISimTrace(1,"Trace Level 1: CallBackManager::RetrieveCallback - Callback Does Not Exist!\n");
        //CallBack does not exist
        return 0; 
    }
    return &callbackIter->second;
}

