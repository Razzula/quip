export function parseMessage(data: string): unknown {
    const message = JSON.parse(data);

    if (!message || typeof message !== 'object') {
        return message;
    }

    const value = message as Record<string, unknown>;

    if ('mute_groups' in value) {
        value.muteGroups = value.mute_groups;
        delete value.mute_groups;
    }

    return value;
}
