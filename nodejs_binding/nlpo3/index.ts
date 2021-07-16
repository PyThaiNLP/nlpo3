import * as nativeModule from './rust_mod'
export const loadDict = (filePath: string, dictName: string): string => {
    return nativeModule.loadDict(filePath, dictName)
}
export const segment = (text: string, dictName = 'default', safe = false, parallel = false): string[] => {
    return nativeModule.segment(text, dictName, safe, parallel)
}