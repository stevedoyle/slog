/**
 * @file CallBackHandler.cpp
 *
 * @date 23 March 2005
 *
 * @brief The CallBackHandler co-ordinates client callbacks.
 * 
 * The CallBackHandler is a singleton. It co-ordinates the synchronous or 
 * asynchronous client callbacks. It uses a callback handle to retrieve the 
 * relevant callback from the callBackManager, it updates data in the callback 
 * and then invokes a member function of the callback with invokes the client 
 * function pointer.
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
#include "CallBackHandler.h"
#include "CallBack.h"
#include "CallBackManager.h"
#include "EventScheduler.h"
#include "TraceMacro.h"

CallBackHandler::CallBackHandler()
: m_eventSchedule(false)
{
}

CallBackHandler::~CallBackHandler()
{
}

CallBackHandler& CallBackHandler::instance()
{
    // Singleton Pattern
    static CallBackHandler instance;
    return instance;
}

/**
 * Function Definition: asyncCallback(NPF_callbackHandle_t cbHandle,
 *                                    NPF_correlator_t     cbCorrelator,
 *                                    NPF_F_ATM_ConfigMgr_CallbackData_t data)
 */
void CallBackHandler::AsyncCallback(NPF_callbackHandle_t cbHandle,
                   NPF_correlator_t     cbCorrelator,
                   NPF_F_ATM_ConfigMgr_CallbackData_t& data)
{    
    APISimTrace(3,"Trace Level 3: CallBackHandler::AsyncCallback(%d,%d,..)\n",cbHandle,cbCorrelator);
    
    CallBack* callbackTemp = CallBackManager::instance().RetrieveCallback(cbHandle);
    
    // If callback does not exist return a failure.
    //if(callbackTemp==0)
    //{
     //  APISimTrace(1,"Trace Level 1: CallBackHandler::AsyncCallback - Callback does not exist!\n");
      // return false;
    //}
    
    // Given the callBackHandle, retrieve the relevant stored callBack entry.
    CallBack* callback = new CallBack(*callbackTemp);
    //callback.m_function = callbackTemp->m_function;

    
    
    // Store client info in callback object
    callback->m_cbCorrelator = cbCorrelator;
    callback->m_data = data;
    
    // Trigger synchronous callback by calling fire function on 
    // retrieved callback entry.
    if(m_eventSchedule == true)
    {
        EventScheduler::instance().addEvent(*callback, 1);        
    }else
    {
        callback->Fire();        
    }
    // Return success.    
}

void CallBackHandler::setCallbackModeAsync()
{
    APISimTrace(3,"Trace Level 3: CallBackHandler::setCallbackModeAsync()\n");
    m_eventSchedule = true;   
}

