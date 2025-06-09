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
#if !defined __TABLEMANAGER_H_
#define __TABLEMANAGER_H_

/**
 * User defined include files required.
 */
#include "npf.h"
#include "NPF_F_ATM_CONFIGURATION_MANAGER.h"

/**
 * Standard defined include files required.
 */
#include <map>
using namespace std;

class TableManager  
{
public:
    virtual ~TableManager();

    static TableManager& instance();
    
    /**
    * @ingroup FAPI Simulator
    *
    * @fn addATMIf(NPF_F_ATM_ConfigMgr_IfCfg_t atmInterface) 
    *
    * @brief Create an ATM interface entry in the storage MAP.
    *
    * @param “atmInterface NPF_F_ATM_ConfigMgr_IfCfg_t [in]” - ATM interface 
    *                                                          attribute 
    *                                                          structure.
    *
    *
    * This function creates an entry in the ATM interface MAP, if a similar entry
    * does not already exist. The ATM inferace ID, stored in the ATM interface
    * structure, is used as the key.
    *
    * @return bool 
    */    
    bool AddATMIf(NPF_F_ATM_ConfigMgr_IfCfg_t* atmInterface, NPF_uint32_t numEntries, NPF_F_ATM_ConfigMgr_CallbackData_t& data);

    bool DeleteIf(NPF_F_ATM_IfID_t *delArray, NPF_uint32_t numEntries, NPF_F_ATM_ConfigMgr_CallbackData_t& data);

    /** 
    * @ingroup FAPI Simulator
    *
    * @fn addATMVC(NPF_F_ATM_ConfigMgr_Vc_t atmVC) 
    *
    * @brief Create an ATM VC entry in the storage MAP.
    *
    * @param “atmVC NPF_F_ATM_ConfigMgr_Vc_t [in]” - ATM VC attribute structure.
    *
    *
    *
    * This function creates an entry in the ATM VC MAP, if a similar entry
    * does not already exist. The ATM VC Link ID, stored in the ATM VC structure, 
    * is used as the key.
    *
    * @return bool 
    */
    bool AddATMVC(NPF_F_ATM_ConfigMgr_Vc_t* atmVC, NPF_uint32_t numEntries, NPF_F_ATM_ConfigMgr_CallbackData_t& data);

    /**
    * @ingroup FAPI Simulator
    *
    * @fn addATMXC(NPF_F_ATM_ConfigMgr_VcLinkXc_t atmXC) 
    *
    * @brief Create an ATM XC entry in the storage MAP.
    *
    * @param “atmXC NPF_F_ATM_ConfigMgr_VcLinkXc_t [in]” - ATM XC attribute structure.
    *
    *
    *
    * This function creates an entry in the ATM XC MAP, if a similar entry
    * does not already exist. The ATM VC Link ID of link A, stored in the ATM XC structure, 
    * is used as the key.
    *
    * @return bool 
    */
    bool AddATMXC(NPF_F_ATM_ConfigMgr_VcLinkXc_t* atmXC, NPF_uint32_t numEntries, NPF_F_ATM_ConfigMgr_CallbackData_t& data);
 
    //test
    void TableManager::printIf();
    void TableManager::printVc();
    void TableManager::printXc();
    
private:
    TableManager();
    TableManager(const TableManager&);
    TableManager& operator =(const TableManager&);

    /**
    * TableManager Member Variables.
    * 
    * ATMInterfaceTable - Map used to store ATM Interface entries, Interface ID 
    *                     is the used as the key.
    * 
    * ATMVCTable - Map used to store VC entries, VCLinkID used as the key.
    *
    */
    map<unsigned int, NPF_F_ATM_ConfigMgr_IfCfg_t> m_ATMInterfaceTable;
    map<unsigned int, NPF_F_ATM_ConfigMgr_Vc_t> m_ATMVCTable;
    map<unsigned int, NPF_F_ATM_ConfigMgr_VcLinkXc_t>m_ATMXCTable;
    /**
    * @ingroup FAPI Simulator
    * 
    * @typedef VCEntry
    *
    * @brief Typedef of a typical entry for the ATM VC MAP
    *
    */    
    typedef pair<unsigned int, NPF_F_ATM_ConfigMgr_Vc_t> VCEntry;

    /**
    * @ingroup FAPI Simulator
    * 
    * @typedef interfaceEntry
    *
    * @brief Typedef of a typical entry for the ATM interface MAP
    *
    */
    typedef pair<unsigned int,NPF_F_ATM_ConfigMgr_IfCfg_t> InterfaceEntry;
    typedef pair<unsigned int,NPF_F_ATM_ConfigMgr_VcLinkXc_t> XCEntry;
    
    typedef map<unsigned int, NPF_F_ATM_ConfigMgr_Vc_t>::iterator VCIterator;
    typedef map<unsigned int, NPF_F_ATM_ConfigMgr_IfCfg_t>::iterator IFIterator;
    typedef map<unsigned int, NPF_F_ATM_ConfigMgr_VcLinkXc_t>::iterator XCIterator;
    
    typedef pair<map<unsigned int, NPF_F_ATM_ConfigMgr_IfCfg_t>::iterator, bool> InterfaceInsertPair;
    typedef pair<map<unsigned int, NPF_F_ATM_ConfigMgr_Vc_t>::iterator, bool> VCInsertPair;
    typedef pair<map<unsigned int, NPF_F_ATM_ConfigMgr_VcLinkXc_t>::iterator, bool> XCInsertPair;

};
#endif // #if !defined __TABLEMANAGER_H_
/**
 *@}
 */
