'use strict';

const chatClient = require('./chat');

(async() => {
    const member = {
        username: 'jsUser',
        password: 'p'
    };

    const newChatMessage = {
        value: 'Message from Node',
        token: 3
    };

    const after = {
        value: 0,
        token: 3
    }

    const r = await chatClient.messages(after);
    console.log(r);
})();