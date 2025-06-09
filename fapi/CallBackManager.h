/**
 * @file CallBackManager.h
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
#if !defined __CALLBACKMANAGER_H_
#define __CALLBACKMANAGER_H_

/**
 * User defined include files required.
 */
#include "npf.h"
#include "NPF_F_ATM_CONFIGURATION_MANAGER.h"
#include "CallBack.h"

/**
 * System defined include files required.
 */
#include <map>
using namespace std;

class CallBackManager  
{
public:
    virtual ~CallBackManager();
    
    static CallBackManager& instance();
    
    /**
    * @ingroup FAPI Simulator
    *
    * @fn registerCallBack(NPF_userContext_t userContext,
    *                      NPF_F_ATM_ConfigMgr_CallBackFunc_t callbackFunc,
    *                      NPF_callbackHandle_t *callbackHandle) 
    *
    * @brief Stores a callback entry and returns a callback handle
    *
    * @param “userContext NPF_userContext_t [in]” - – The context item that was
    *                                                 supplied by the 
    *                                                 application when the 
    *                                                 completion callback 
    *                                                 function was registered.
    * @param “callbackFunc NPF_F_ATM_ConfigMgr_CallBackFunc_t [in]” - The 
    *                                                                 pointer 
    *                                                                 to the 
    *                                                                 completion
    *                                                                 callback 
    *                                                                 function 
    *                                                                 to be 
    *                                                                 registered.
    * @param “*callbackHandle NPF_callbackHandle_t [in]” - A unique identifier 
    *                                                      assigned for the 
    *                                                      registered 
    *                                                      atmUserContext and 
    *                                                      atmEventCallFunc pair.  
    *
    *
    * This operation stores a callback entry using a unique key which is passed
    * back to the client and refered to as the callback handle. The operation
    * instanciates a callback entry class to store the usercontext and 
    * callbackfunc for a given key. If a identical usercontext and callbackfunc
    * pair are already stored the existing key will be returned with a failure.
    *
    * @return bool
    */
    NPF_error_t RegisterCallBack(NPF_userContext_t userContext,
                          NPF_F_ATM_ConfigMgr_CallBackFunc_t callbackFunc,
                          NPF_callbackHandle_t *callbackHandle);
                          
    NPF_error_t DeRegisterCallBack(NPF_callbackHandle_t cbHandle);
    /**
    * @ingroup FAPI Simulator
    *
    * @fn retrieveCallback(NPF_callbackHandle_t cbHandle) 
    *
    * @brief Retrieves registered callback information.
    *
    * @param “cbHandle NPF_callbackHandle_t [in]” - A unique identifier assigned
    *                                               for the registered 
    *                                               atmUserContext and 
    *                                               atmEventCallFunc pair.
    *
    *
    * Given a valid callback handle the operation returns the relevant stored
    * callback entry. The callback entry contains a unique usercontext, 
    * callbackfunc pair.
    *
    * @return CallBack
    */    
    CallBack* RetrieveCallback(NPF_callbackHandle_t cbHandle);
    
    private:
    CallBackManager();
    CallBackManager(const CallBackManager&);
    CallBackManager& operator =(const CallBackManager&);

    /**
    * @ingroup FAPI Simulator
    * 
    * @typedef callbackEntry
    *
    * @brief Typedef of a typical entry for the callback MAP
    *
    */
    typedef pair<unsigned int, CallBack> callbackEntry;
    
    /**
    * @ingroup FAPI Simulator
    * 
    * @typedef callBackTable
    *
    * @brief Typedef of a callBackTable
    *
    */
    typedef map<unsigned int, CallBack> callBackTable;

    /**
    * @ingroup FAPI Simulator
    * 
    * @typedef tableIterator
    *
    * @brief Typedef of callBackTable iterator
    *
    */
    typedef map<unsigned int, CallBack>::const_iterator tableIterator;         

    /**
    * @ingroup FAPI Simulator
    * 
    * @typedef tableFindIterator
    *
    * @brief Typedef of callBackTable find iterator
    *
    */
    typedef map<unsigned int, CallBack>::iterator tableFindIterator;        
        
    /**
    * CallBackManager Member Variables.
    * 
    * m_handleID - A unique identifier assigned for the registered 
    *              atmUserContext and atmEventCallFunc pair.
    * 
    * m_callbackTable - Map containing callback entries identified by a 
    *                   callback handle key.
    *
    */
    unsigned int m_handleID;
    callBackTable m_callbackTable;


};
#endif // #if !defined __CALLBACKMANAGER_H_
/**
 *@}
 */
