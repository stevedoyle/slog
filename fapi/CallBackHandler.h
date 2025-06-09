/**
 * @file CallBackHandler.h
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
 * Design Notes:
 *    <describe any non-obvious design decisions which impact this file>
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

/**
 * @defgroup FAPI Simulator
 *
 * @brief FAPI Simulator mimics the behaviour of the control plane interface,
 *             by a client, to the FWM product, through standard NPF APIs.
 *
 * @{
 */
#if !defined __CALLBACKHANDLER_H_
#define __CALLBACKHANDLER_H_

/**
 * User defined include files required.
 */
#include "npf.h"
#include "NPF_F_ATM_CONFIGURATION_MANAGER.h"

class CallBackHandler  
{
public:
    virtual ~CallBackHandler();
    
    static CallBackHandler& instance();

    /**
    * @ingroup FAPI Simulator
    *
    * @fn asyncCallback(NPF_callbackHandle_t cbHandle,
    *                   NPF_correlator_t cbCorrelator,
    *                   NPF_F_ATM_ConfigMgr_CallbackData_t data) 
    *
    * @brief Co-Ordination of client callback invocation.
    *
    * @param “cbHandle NPF_callbackHandle_t [in]” - Used to retrieve the 
    *                                               required callback from the 
    *                                               callBackManager.
    * @param “cbCorrelator NPF_correlator_t [in]” - Returned to Client for 
    *                                               identification of callback 
    *                                               instance.
    * @param “data NPF_F_ATM_ConfigMgr_CallbackData_t [in]” - Data structure 
    *                                                         containing
    *                                                         information 
    *                                                         related to 
    *                                                         the finished
    *                                                         FAPI call.
    *
    *
    *
    * This function uses a callback handle to retrieve a relevant callback 
    * from the callBackManager, it updates data in the callback and then invokes
    * a member function of the callback with invokes the client function pointer.
    *
    * @return bool 
    */
    void AsyncCallback(NPF_callbackHandle_t cbHandle,
                       NPF_correlator_t cbCorrelator,
                       NPF_F_ATM_ConfigMgr_CallbackData_t& data);
                       
    void setCallbackModeAsync();
                       
private:
    CallBackHandler();
    CallBackHandler(const CallBackHandler&);
    CallBackHandler& operator =(const CallBackHandler&);
    
    bool m_eventSchedule;
    
};
#endif // #if !defined __CALLBACKHANDLER_H_
/**
 *@}
 */

