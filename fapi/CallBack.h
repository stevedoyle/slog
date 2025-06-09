/**
 * @file CallBack.h
 *
 * @date 23 March 2005
 *
 * @brief Stores Client data relating to function call back registration by a 
 * FAPI Sim client.
 *
 * An instance of this class is created to store userContext, callback function 
 * pointer, callback correlator and callback data per client registered 
 * callback. The object also invokes the callback function pointer with relevant
 * parameters.
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
 
 /**
 * @defgroup FAPI Simulator 
 *
 * @brief FAPI Simulator mimics the behaviour of the control plane interface,
 *        by a client, to the FWM product, through standard NPF APIs.
 *
 * @{
 */

#if !defined _CALLBACK_H_
#define _CALLBACK_H_

/**
 * User defined include files required.
 */
#include "npf.h"
#include "NPF_F_ATM_CONFIGURATION_MANAGER.h"
#include "Event.h"

class CallBack : public Event
{
public:
    CallBack(NPF_userContext_t userContext,
             NPF_F_ATM_ConfigMgr_CallBackFunc_t callbackFunc);           
    virtual ~CallBack();
    
    /**
     * CallBack Member Variables.
     * 
     * m_context - The context item that was supplied by the 
     *             application when the completion callback 
     *             function was registered.
     * 
     * m_function - The pointer to the completion callback function
     *              to be registered.
     * 
     * m_cbCorrelator - A unique application invocation value that 
     *                  will be supplied to the asynchronous completion
     *                  callback routine.
     * 
     * m_data - Response information related to the function call. 
     *          Contains information that is common among all functions,
     *          as well as information that is specific to a particular 
     *          function.
    */
    NPF_userContext_t m_context;
    NPF_F_ATM_ConfigMgr_CallBackFunc_t m_function;
    NPF_correlator_t m_cbCorrelator;
    NPF_F_ATM_ConfigMgr_CallbackData_t m_data;
    
    /**
    * @ingroup FAPI Simulator
    *
    * @fn fire() 
    *
    * @brief The fire() operation invokes the registered callback function 
    *        with the relevant parameters.
    *
    *
    * Global Data  : None
    *
    * @return None
    */
    virtual void Fire();
    
};
#endif //#if !defined _CALLBACK_H_
/**
 *@}
 */

