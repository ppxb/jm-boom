import { apiClient } from './client'

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
  user: UserProfile
}

export type SignInRecord = {
  day: number
  date: string
  signed: boolean
  bonus: boolean
}

export type SignInDataResult = {
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
  message: string
}

export async function login({
  username,
  password
}: {
  username: string
  password: string
}): Promise<LoginResult> {
  return apiClient.post('/api/auth/login', { username, password })
}

export async function getCurrentSession(): Promise<LoginResult | null> {
  return apiClient.get('/api/auth/session')
}

export async function getSignInData({ userId }: { userId: number }): Promise<SignInDataResult> {
  return apiClient.get('/api/auth/sign-in', { userId })
}

export async function signIn({
  userId,
  dailyId
}: {
  userId: number
  dailyId: number
}): Promise<SignInResult> {
  return apiClient.post('/api/auth/sign-in', { userId, dailyId })
}

export async function clearSession(): Promise<void> {
  await apiClient.delete('/api/auth/session')
}
