import type { ReactNode } from 'react'

import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue
} from '@/components/ui/select'
import { cn } from '@/lib/utils'

type FilterSelectOption = {
  label: string
  value: string
}

type FilterSelectProps = {
  value: string
  options: readonly FilterSelectOption[]
  placeholder: string
  icon?: ReactNode
  grow?: boolean
  onValueChange: (value: string) => void
}

export function FilterSelect({
  value,
  options,
  placeholder,
  icon,
  grow = true,
  onValueChange
}: FilterSelectProps) {
  return (
    <div className={cn('min-w-0 sm:flex-none', grow ? 'flex-1' : 'flex-none')}>
      <Select value={value} onValueChange={onValueChange}>
        <SelectTrigger className={grow ? 'w-full sm:w-auto' : 'w-auto'}>
          {icon}
          <SelectValue placeholder={placeholder} />
        </SelectTrigger>
        <SelectContent position="popper" align="end">
          <SelectGroup>
            {options.map(option => (
              <SelectItem key={option.value} value={option.value}>
                {option.label}
              </SelectItem>
            ))}
          </SelectGroup>
        </SelectContent>
      </Select>
    </div>
  )
}
