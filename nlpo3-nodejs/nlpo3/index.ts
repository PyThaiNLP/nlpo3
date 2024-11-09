// SPDX-FileCopyrightText: 2024 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

import * as nativeModule from './rust_mod'
/**
 * Load dict from dictionary file and store in hash map with key = dictName for ***segment*** function to use.
 * 
 * filePath is an absolute path to the dictionary file.
 */
export const loadDict = (filePath: string, dictName: string): string => {
    return nativeModule.loadDict(filePath, dictName)
}
/**
 * Perform segmentation on "text" argument with words from dict "dictName".
 * 
 * Dictionary "dictName" must be loaded with **loadDict** function first.
 * 
 */
export const segment = (text: string, dictName: string, safe = false, parallel = false): string[] => {
    return nativeModule.segment(text, dictName, safe, parallel)
}
