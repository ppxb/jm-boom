import { Trash2Icon, XIcon } from 'lucide-react'
import type { ReactNode } from 'react'

import { ConfirmDialog } from '@/components/confirm-dialog'
import { Button } from '@/components/ui/button'

type SelectionActionsProps = {
  isSelecting: boolean
  allSelected: boolean
  selectedCount: number
  disabled?: boolean
  loading?: boolean
  enterLabel: string
  enterIcon?: ReactNode
  deleteLabel?: string
  dialogTitle: string
  dialogDescription: ReactNode
  confirmText?: string
  idleActions?: ReactNode
  onEnter: () => void
  onExit: () => void
  onToggleAll: () => void
  onDeleteSelected: () => void
}

export function SelectionActions({
  isSelecting,
  allSelected,
  selectedCount,
  disabled = false,
  loading = false,
  enterLabel,
  enterIcon,
  deleteLabel = '删除选中',
  dialogTitle,
  dialogDescription,
  confirmText = '确认删除',
  idleActions,
  onEnter,
  onExit,
  onToggleAll,
  onDeleteSelected
}: SelectionActionsProps) {
  if (!isSelecting) {
    return (
      <>
        <Button type="button" variant="outline" size="sm" disabled={disabled} onClick={onEnter}>
          {enterIcon}
          {enterLabel}
        </Button>
        {idleActions}
      </>
    )
  }

  return (
    <>
      <Button
        type="button"
        variant="outline"
        size="sm"
        disabled={disabled || loading}
        onClick={onToggleAll}
      >
        {allSelected ? '取消全选' : '全选'}
      </Button>
      <ConfirmDialog
        trigger={
          <Button
            type="button"
            variant="destructive"
            size="sm"
            disabled={selectedCount === 0 || loading}
          >
            <Trash2Icon className="size-4" />
            {deleteLabel}
          </Button>
        }
        icon={<Trash2Icon className="size-5 text-destructive" />}
        title={dialogTitle}
        description={dialogDescription}
        confirmText={confirmText}
        variant="destructive"
        loading={loading}
        onConfirm={onDeleteSelected}
      />
      <Button type="button" variant="ghost" size="sm" disabled={loading} onClick={onExit}>
        <XIcon className="size-4" />
        退出
      </Button>
    </>
  )
}
