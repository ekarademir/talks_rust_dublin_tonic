'use strict';

const { ArgumentParser } = require('argparse');

const chatClient = require('./chat');

(async() => {

    const parser = new ArgumentParser({
        description: 'Chat client',
        prog: 'CHAT'
    });

    const subparsers = parser.add_subparsers({
        dest: 'command'
    });

    const joinCommand = subparsers.add_parser('join', {
        help: 'Join a chat server'
    });
    joinCommand.add_argument('-s', '--server', {
        default: 'localhost:10000'
    });
    joinCommand.add_argument('-u', '--username', {
        required: true
    });
    joinCommand.add_argument('-p', '--password', {
        required: true
    });

    const sendCommand = subparsers.add_parser('send', {
        help: 'Send a message to a joined server'
    });
    sendCommand.add_argument('-s', '--server', {
        default: 'localhost:10000'
    });
    sendCommand.add_argument('-t', '--token', {
        required: true
    });
    sendCommand.add_argument('-m', '--message', {
        required: true
    });

    const messagesCommand = subparsers.add_parser('messages', {
        help: 'download all the messages from given time onwards'
    });
    messagesCommand.add_argument('-s', '--server', {
        default: 'localhost:10000'
    });
    messagesCommand.add_argument('-t', '--token', {
        required: true
    });
    messagesCommand.add_argument('-a', '--after', {
        default: 0
    });

    const args = parser.parse_args();

    if (args.command && args.command === 'join') {
        console.log(await chatClient.join({...args }));
        return;
    }
    if (args.command && args.command === 'send') {
        console.log(await chatClient.send({...args }));
        return;
    }
    if (args.command && args.command === 'messages') {
        console.log(await chatClient.messages({...args }));
        return;
    }
})();