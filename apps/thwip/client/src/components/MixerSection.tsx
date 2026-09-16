import type { ReactNode } from 'react'
import './MixerSection.scss'

interface MixerSectionProps {
    title: string
    description?: string
    children: ReactNode
}

export function MixerSection({
    title,
    description,
    children,
}: MixerSectionProps) {
    return (
        <section className="mixer-section">
            <div className="mixer-section__header">
                <h2>{title}</h2>
                <span>{description}</span>
            </div>

            {children}
        </section>
    )
}
