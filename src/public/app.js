document.querySelectorAll('.lobby.panel .mode').forEach((elem) => {
     elem.addEventListener('click', () => {
         if (elem.classList.contains('selected')) {
             return;
         }
         document.querySelector('.lobby.panel .mode.selected').classList.remove('selected');

         elem.classList.add('selected');
         document.querySelector('.lobby.panel .input.username').style.display = elem.classList.contains('host') ? 'none' : 'block';
         document.querySelector('.lobby.panel .action').innerText = `${elem.classList.contains('host') ? 'Create' : 'Join'} room`;
         if (elem.classList.contains('host')) {
             if (document.querySelector('.lobby.panel .input.room > input').value.trim().length === 0) {
                 document.querySelector('.lobby.panel .input.room > input').value = `Room ${1000 + Math.floor(Math.random() * 9000)}`;
                 document.querySelector('.lobby.panel .input.room > input').focus();
                 document.querySelector('.lobby.panel .input.room > input').select();
             }
         } else {
             if (/Room \d{4}/.test(document.querySelector('.lobby.panel .input.room > input').value.trim())) {
                 document.querySelector('.lobby.panel .input.room > input').value = '';
                 document.querySelector('.lobby.panel .input.room > input').focus();
             }
         }
     })
});

if (localStorage.getItem('username') !== null) {
    document.querySelector('.lobby.panel .input.username > input').value = localStorage.getItem('username');
}
document.querySelector('.lobby.panel .input.username > input').addEventListener('change', (event) => {
    localStorage.setItem('username', event.target.value.trim());
});

const roomPlaceholder = new URLSearchParams(window.location.search).get('room')?.trim();
if (roomPlaceholder) {
    document.querySelector('.lobby.panel .input.room > input').value = roomPlaceholder;
    document.querySelector('.lobby.panel .input.username > input').focus();
    document.querySelector('.lobby.panel .input.username > input').select();
} else {
    document.querySelector('.lobby.panel .input.room > input').focus();
    document.querySelector('.lobby.panel .input.room > input').select();
}

function proceed(event) {
    event.preventDefault();

    (async function() {
        if (document.querySelector('.lobby.panel .mode.selected').classList.contains('host')) {
            const response = await fetch("/rooms", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    name: document.querySelector('.lobby.panel .room.input > input').value.trim(),
                }),
            });
            let data = await response.json();
            if (data.error) {
                alert(`${data.error}.`);
                return;
            }
            let { id, name } = data;

            run('host', name, new WebSocket(`${location.origin.replace(/^http/, 'ws')}/rooms/${id}/host`), document.querySelector('.host.panel'));
        } else {
            const response = await fetch(`/rooms/id?name=${document.querySelector('.lobby.panel .room.input > input').value.trim()}`, {
                method: 'GET',
            });
            let data = await response.json();
            if (data.error) {
                alert(`${data.error}.`);
                return;
            }
            let { id, name } = data;

            run('participate', name, new WebSocket(`${location.origin.replace(/^http/, 'ws')}/rooms/${id}/participate?name=${document.querySelector('.lobby.panel .username.input > input').value.trim()}`), document.querySelector('.participate.panel'));
        }
    })();
}

document.querySelector('.lobby.panel .action').addEventListener('click', proceed);
document.querySelector('.lobby.panel form').addEventListener('submit', proceed);

function run(mode, name, socket, panelElem) {
    document.body.classList.replace('lobby', mode);
    panelElem.querySelector('.title.panel > .labels > .label').innerText = name;

    const url = new URL(window.location);
    url.searchParams.set('room', name);
    window.history.replaceState(null, '', url.toString());

    let buzzs = [];
    let buzzed = false;
    let initiatedLeave = false;

    socket.addEventListener('message', (message) => {
        const data = JSON.parse(message.data);
        switch (data.event) {
            case 'participantCount':
                panelElem.querySelector('.title.panel > .labels > .sub-label').innerText = `${data.count} participant${data.count !== 1 ? 's' : ''}`;
                break;
            case 'buzzed':
                const { id, name, timestampDiff } = data;

                const buzzElem = document.createElement('div');
                buzzElem.classList.add('buzz');
                if (buzzs.length === 0) {
                    buzzElem.classList.add('selected');
                }

                const usernameElem = document.createElement('div');
                usernameElem.classList.add('username', 'selectable');
                usernameElem.innerText = name;
                usernameElem.title = name;
                buzzElem.append(usernameElem);

                const rightElem = document.createElement('div');
                rightElem.classList.add('right');
                buzzElem.append(rightElem);

                const positionElem = document.createElement('div');
                positionElem.classList.add('position');
                switch (buzzs.length + 1) {
                    case 1:
                        positionElem.innerText = '1st';
                        break;
                    case 2:
                        positionElem.innerText = '2nd';
                        break;
                    case 3:
                        positionElem.innerText = '3rd';
                        break;
                    default:
                        positionElem.innerText = `${buzzs.length + 1}th`;
                        break;
                }
                rightElem.append(positionElem);

                if (timestampDiff !== null) {
                    const timestampElem = document.createElement('div');
                    timestampElem.classList.add('timestamp');
                    if (timestampDiff < 1000) {
                        timestampElem.innerText = `+${timestampDiff}ms`;
                    } else if (timestampDiff < 10000) {
                        timestampElem.innerText = `+${Math.floor(timestampDiff / 100) / 10}s`;
                    } else {
                        timestampElem.innerText = `+${Math.floor(timestampDiff / 1000)}s`;
                    }
                    rightElem.append(timestampElem);
                }

                panelElem.querySelector('.inner.panel').append(buzzElem);
                buzzs.push({ id, elem: buzzElem });
                break;
            case 'select':
                switch (mode) {
                    case 'host':
                        panelElem.querySelector('.inner.panel .buzz.selected').classList.remove('selected');
                        const buzzElem = buzzs.find((b) => b.id === data.id).elem;
                        buzzElem.classList.add('selected');
                        buzzElem.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
                        break;
                    case 'participate':
                        panelElem.querySelector('.inner.panel').classList.remove('waiting');
                        panelElem.querySelector('.inner.panel').classList.add('selected');
                        break;
                }
                break;
            case 'deselect':
                panelElem.querySelector('.inner.panel').classList.remove('selected');
                break;
            case 'clear':
                panelElem.querySelector('.inner.panel').classList.remove('selected', 'waiting');
                buzzed = false;
                break;
            case 'hostLeft':
                exit();
                alert('The host has closed the room.');
                break;
        }
    });

    socket.addEventListener('error', () => {
        if (!initiatedLeave) {
            alert('An error occurred');
        }
        exit();
    });

    function handleBuzz() {
        socket.send(JSON.stringify({ event: 'buzz' }));
        if (!buzzed) {
            buzzed = true;
            panelElem.querySelector('.inner.panel').classList.add('waiting');
        }
    }

    function handleLeave() {
        initiatedLeave = true;
        if (mode === 'participate' || confirm('You\'re about to close the room and kick every participant out. Confirm?')) {
            exit();
        }
    }

    function handleClear() {
        socket.send(JSON.stringify({ event: 'clear' }));
        buzzs = [];
        panelElem.querySelector('.inner.panel').replaceChildren();
    }

    function handleSelect() {
        socket.send(JSON.stringify({ event: 'selectNext' }));
    }

    function keyDown(event) {
        switch (mode) {
            case 'host':
                switch (event.key) {
                    case 'Backspace':
                    case 'Escape':
                        event.preventDefault();
                        handleClear();
                        break;
                    case 'ArrowDown':
                        handleSelect();
                        break;
                }
                break;
            case 'participate':
                if (event.key === ' ') {
                    handleBuzz();
                    panelElem.querySelector('.buzzer').classList.add('buzzing');
                }
                break;
        }
    }

    function keyUp(event) {
        switch (mode) {
            case 'participate':
                if (event.key === ' ') {
                    panelElem.querySelector('.buzzer').classList.remove('buzzing');
                }
                break;
        }
    }

    function exit() {
        socket.close();
        panelElem.querySelectorAll('.inner.panel .buzz').forEach((elem) => elem.remove());
        panelElem.querySelector('.inner.panel').classList.remove('selected', 'waiting');
        panelElem.querySelector('.buzzer')?.removeEventListener('click', handleBuzz);
        panelElem.querySelector('.leave.action')?.removeEventListener('click', handleLeave);
        panelElem.querySelector('.clear.action')?.removeEventListener('click', handleClear);
        panelElem.querySelector('.select.action')?.removeEventListener('click', handleSelect);
        window.removeEventListener('keydown', keyDown);
        window.removeEventListener('keyup', keyUp);

        const url = new URL(window.location);
        url.searchParams.delete('room');
        window.history.replaceState(null, '', url.toString());
        document.body.classList.replace(mode, 'lobby');
    }

    panelElem.querySelector('.buzzer')?.addEventListener('click', handleBuzz);
    panelElem.querySelector('.leave.action')?.addEventListener('click', handleLeave);
    panelElem.querySelector('.clear.action')?.addEventListener('click', handleClear);
    panelElem.querySelector('.select.action')?.addEventListener('click', handleSelect);
    window.addEventListener('keydown', keyDown);
    window.addEventListener('keyup', keyUp);
}