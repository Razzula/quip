interface MixerSectionProps {
    title: string
    children: React.ReactNode
    className?: string
}

export function MixerSection({
    title,
    children,
    className = '',
}: MixerSectionProps) {
    return (
        <section className={`mixer-section ${className}`}>
            <header className="mixer-section__header">
                <h2>{title}</h2>
            </header>

            {children}
        </section>
    )
}
