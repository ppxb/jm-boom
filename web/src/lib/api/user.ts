// import { apiClient } from './client'

export type UserProfile = {
  id: number
  username: string
  email: string
  avatar: string
  avatarUrl: string
  level: number
  levelName: string
  currentLevelExp: number
  nextLevelExp: number
  expPercent: number
  currentCollectCount: number
  maxCollectCount: number
  jCoin: number
}

export type LoginResult = {
  endpoint: string
  user: UserProfile
}

export type SavedLoginConfig = {
  endpoint: string
  username: string
  autoLogin: boolean
  hasPassword: boolean
}

export type SignInRecord = {
  day: number
  date: string
  signed: boolean
  bonus: boolean
}

export type SignInDataResult = {
  endpoint: string
  dailyId: number
  threeDaysCoin: number
  threeDaysExp: number
  sevenDaysCoin: number
  sevenDaysExp: number
  eventName: string
  currentProgress: string
  backgroundPc: string
  backgroundPhone: string
  records: SignInRecord[]
}

export type SignInResult = {
  endpoint: string
  message: string
}

export async function login({
  username: _username,
  password: _password,
  endpoint: _endpoint = null,
  rememberLogin: _rememberLogin = false
}: {
  username: string
  password: string
  endpoint?: string | null
  rememberLogin?: boolean
}): Promise<LoginResult> {
  // TODO: 实现后端登录 API
  throw new Error('Login not implemented in HTTP mode')
}

export async function getCurrentSession(): Promise<LoginResult | null> {
  // TODO: 实现后端会话查询 API
  return null
}

export async function getSavedLoginConfig(): Promise<SavedLoginConfig | null> {
  // HTTP 模式不支持本地保存
  return null
}

export async function saveLoginCredentials({
  username: _username,
  password: _password,
  endpoint: _endpoint = null,
  autoLogin: _autoLogin
}: {
  username: string
  password: string
  endpoint?: string | null
  autoLogin: boolean
}): Promise<SavedLoginConfig> {
  // HTTP 模式不支持本地保存
  throw new Error('Credential storage not supported in HTTP mode')
}

export async function setLoginAutoLogin(_autoLogin: boolean): Promise<SavedLoginConfig | null> {
  return null
}

export async function clearLoginCredentials(): Promise<void> {
  // HTTP 模式不支持本地保存
}

export async function getSignInData({
  userId: _userId,
  endpoint: _endpoint = null
}: {
  userId: number
  endpoint?: string | null
}): Promise<SignInDataResult> {
  // TODO: 实现后端签到数据 API
  throw new Error('Sign-in not implemented in HTTP mode')
}

export async function signIn({
  userId: _userId,
  dailyId: _dailyId,
  endpoint: _endpoint = null
}: {
  userId: number
  dailyId: number
  endpoint?: string | null
}): Promise<SignInResult> {
  // TODO: 实现后端签到 API
  throw new Error('Sign-in not implemented in HTTP mode')
}

export async function clearSession(): Promise<void> {
  // TODO: 实现后端登出 API
}
