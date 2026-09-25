export interface RoleplayObjective {
    id: string;
    description: string;
}

export interface RoleplayPhrase {
    target: Record<string, string>;
    native: string;
}

export interface RoleplayPersona {
    id: string;
    title: string;
    personaName: string;
    role: string;
    category: 'dining' | 'shopping' | 'travel' | 'services' | 'custom';
    difficulty: 'A1' | 'A2' | 'B1' | 'B2';
    vrmModel: 'avatar.vrm' | 'man.vrm';
    icon: string;
    color: string;
    badgeColor: string;
    setting: string;
    summary: string;
    scenarioPrompt: string;
    initialGreeting: Record<string, string>;
    initialGreetingNative: string;
    suggestedPhrases: RoleplayPhrase[];
    objectives: RoleplayObjective[];
}

export const ROLEPLAY_SCENARIOS: RoleplayPersona[] = [
    {
        id: 'ordering_coffee',
        title: 'The Cozy Café',
        personaName: 'Camille / Min-ji',
        role: 'Friendly Café Barista',
        category: 'dining',
        difficulty: 'A1',
        vrmModel: 'avatar.vrm',
        icon: '☕',
        color: 'from-amber-500/20 to-orange-500/10 border-amber-500/30 text-amber-300',
        badgeColor: 'bg-amber-500/10 text-amber-400 border-amber-500/20',
        setting: 'A cozy corner café with the warm aroma of espresso beans and soft acoustic music.',
        summary: 'Practice ordering your favorite hot or iced beverage, choosing milk options, ordering pastries, and paying.',
        scenarioPrompt: `You are Camille (or Min-ji), an enthusiastic and welcoming barista at a local specialty coffee shop.
The user is a customer ordering drinks and snacks.
Your responsibilities:
- Greet the customer warmly and ask what they would like to drink.
- Ask clarifying questions naturally (hot or iced, drink size, milk choice like whole/oat/almond, sugar).
- Offer a freshly baked pastry (croissant, muffin, scone) to pair with their drink.
- Give a realistic price when they are ready to pay and ask how they'd like to pay (card or cash).
- Conclude with a warm goodbye and wish them a great day.`,
        initialGreeting: {
            french: 'Bonjour ! Bienvenue au Café des Fleurs. Qu\'est-ce qui vous ferait plaisir aujourd\'hui ?',
            korean: '어서오세요! 파를레 카페입니다. 오늘 어떤 음료로 준비해 드릴까요?',
            spanish: '¡Hola! Bienvenido al Café Central. ¿Qué te gustaría tomar hoy?',
            german: 'Hallo! Willkommen im Café Sonne. Was darf ich Ihnen heute bringen?',
            japanese: 'いらっしゃいませ！カフェへようこそ。今日は何になさいますか？',
            russian: 'Здравствуйте! Добро пожаловать в наше уютное кафе. Что для вас приготовить сегодня?',
            ukrainian: 'Доброго дня! Ласкаво просимо до нашої кав\'ярні. Що для вас приготувати сьогодні?',
            english: 'Hello! Welcome to the café. What can I get started for you today?'
        },
        initialGreetingNative: 'Hello! Welcome to the café. What can I get started for you today?',
        suggestedPhrases: [
            {
                target: {
                    french: 'Je voudrais un café au lait, s\'il vous plaît.',
                    korean: '따뜻한 카페라떼 한 잔 주세요.',
                    spanish: 'Quisiera un café con leche, por favor.',
                    german: 'Ich hätte gerne einen Milchkaffee, bitte.',
                    japanese: 'カフェラテを一つお願いします。',
                    russian: 'Я хотел бы кофе с молоком, пожалуйста.',
                    ukrainian: 'Я хотів би каву з молоком, будь ласка.',
                    english: 'I would like a coffee with milk, please.'
                },
                native: 'I would like a coffee with milk, please.'
            },
            {
                target: {
                    french: 'Est-ce que vous avez du lait d\'avoine ?',
                    korean: '혹시 오트밀크(귀리 우유)로 바꿀 수 있나요?',
                    spanish: '¿Tienen leche de avena?',
                    german: 'Haben Sie Hafermilch?',
                    japanese: 'オーツミルクに変更できますか？',
                    russian: 'У вас есть овсяное молоко?',
                    ukrainian: 'У вас є вівсяне молоко?',
                    english: 'Do you have oat milk?'
                },
                native: 'Do you have oat milk?'
            },
            {
                target: {
                    french: 'Je vais prendre aussi un croissant chaud.',
                    korean: '따뜻한 크루아상도 하나 같이 주세요.',
                    spanish: 'También me llevo un cruasán caliente.',
                    german: 'Ich nehme auch ein warmes Croissant.',
                    japanese: '温かいクロワッサンも一つください。',
                    russian: 'Я также возьму теплый круассан.',
                    ukrainian: 'Я також візьму теплий круасан.',
                    english: 'I\'ll also take a warm croissant.'
                },
                native: 'I\'ll also take a warm croissant.'
            },
            {
                target: {
                    french: 'Combien ça coûte ? Je peux payer par carte ?',
                    korean: '얼마인가요? 카드로 결제할게요.',
                    spanish: '¿Cuánto cuesta? ¿Puedo pagar con tarjeta?',
                    german: 'Wie viel kostet das? Kann ich mit Karte zahlen?',
                    japanese: 'いくらですか？カードで払えますか？',
                    russian: 'Сколько это стоит? Можно оплатить картой?',
                    ukrainian: 'Скільки це коштує? Чи можу я розрахуватися карткою?',
                    english: 'How much is it? Can I pay by card?'
                },
                native: 'How much is it? Can I pay by card?'
            }
        ],
        objectives: [
            { id: 'greet_order', description: 'Greet the barista and order a beverage' },
            { id: 'customize_drink', description: 'Specify size, temperature (hot/iced), or milk type' },
            { id: 'snack_pastry', description: 'Ask about or order a pastry or snack' },
            { id: 'pay_checkout', description: 'Ask for the total price, pay, and say farewell' }
        ]
    },
    {
        id: 'ordering_food',
        title: 'Le Petit Bistro',
        personaName: 'Chef Pierre / Jin-woo',
        role: 'Attentive Restaurant Waiter',
        category: 'dining',
        difficulty: 'A2',
        vrmModel: 'man.vrm',
        icon: '🍽️',
        color: 'from-rose-500/20 to-pink-500/10 border-rose-500/30 text-rose-300',
        badgeColor: 'bg-rose-500/10 text-rose-400 border-rose-500/20',
        setting: 'A lively traditional bistro with white tablecloths, candle-lit tables, and ambient clinking glasses.',
        summary: 'Request a table, ask for chef recommendations, order courses and drinks, and ask for the check.',
        scenarioPrompt: `You are Pierre (or Jin-woo), an attentive, polite waiter at a traditional neighborhood restaurant/bistro.
The user is a patron dining at your restaurant.
Your responsibilities:
- Welcome the patron and seat them at a nice table.
- Hand them the menu and ask if they would like an aperitif or water to begin.
- Describe the dish of the day / chef's special with appetizing details when asked.
- Take their appetizer and main dish order, accommodating any dietary questions.
- Check in midway through the meal to ensure everything tastes wonderful.
- Bring the bill when requested, handle payment, and wish them a wonderful evening.`,
        initialGreeting: {
            french: 'Bonsoir ! Bienvenue au Petit Bistro. Vous avez une réservation, ou désirez-vous une table pour deux ?',
            korean: '안녕하세요! 르 쁘띠 비스트로에 오신 것을 환영합니다. 예약하셨나요, 아니면 두 분 테이블로 안내해 드릴까요?',
            spanish: '¡Buenas tardes! Bienvenidos a nuestro restaurante. ¿Tienen reserva o prefieren una mesa para dos?',
            german: 'Guten Abend! Willkommen im Restaurant. Haben Sie reserviert, oder möchten Sie einen Tisch für zwei?',
            japanese: 'いらっしゃいませ！レストランへようこそ。ご予約はございますか、それとも2名様席をご希望ですか？',
            russian: 'Добрый вечер! Добро пожаловать в наш ресторан. Вы бронировали столик, или вас проводить за столик на двоих?',
            ukrainian: 'Доброго вечора! Ласкаво просимо до нашого ресторану. Ви бронювали столик чи бажаєте столик на двох?',
            english: 'Good evening! Welcome to the bistro. Do you have a reservation, or would you like a table for two?'
        },
        initialGreetingNative: 'Good evening! Welcome to the bistro. Do you have a reservation, or would you like a table for two?',
        suggestedPhrases: [
            {
                target: {
                    french: 'Une table pour deux près de la fenêtre, s\'il vous plaît.',
                    korean: '창가 쪽 두 명 테이블로 부탁드립니다.',
                    spanish: 'Una mesa para dos cerca de la ventana, por favor.',
                    german: 'Einen Tisch für zwei am Fenster, bitte.',
                    japanese: '窓際の2名席をお願いします。',
                    russian: 'Столик на двоих у окна, пожалуйста.',
                    ukrainian: 'Столик на двох біля вікна, будь ласка.',
                    english: 'A table for two near the window, please.'
                },
                native: 'A table for two near the window, please.'
            },
            {
                target: {
                    french: 'Quel est le plat du jour recommandé par le chef ?',
                    korean: '오늘 셰프 추천 요리나 오늘의 메뉴는 무엇인가요?',
                    spanish: '¿Cuál es el plato del día que recomienda el chef?',
                    german: 'Was ist das Tagesgericht, das der Küchenchef empfiehlt?',
                    japanese: 'シェフのおすすめの今日の日替わり料理は何ですか？',
                    russian: 'Какое блюдо дня рекомендует шеф-повар?',
                    ukrainian: 'Яка страва дня рекомендована шеф-кухарем?',
                    english: 'What is the chef\'s recommended dish of the day?'
                },
                native: 'What is the chef\'s recommended dish of the day?'
            },
            {
                target: {
                    french: 'C\'est délicieux ! Pourriez-vous nous apporter l\'addition ?',
                    korean: '정말 맛있었습니다! 계산서 좀 가져다주시겠어요?',
                    spanish: '¡Estuvo delicioso! ¿Nos podría traer la cuenta, por favor?',
                    german: 'Es war köstlich! Könnten Sie uns bitte die Rechnung bringen?',
                    japanese: 'とても美味しかったです！お会計をお願いできますか？',
                    russian: 'Было очень вкусно! Не могли бы вы принести счет?',
                    ukrainian: 'Було дуже смачно! Чи не могли б ви принести рахунок?',
                    english: 'It was delicious! Could you please bring us the bill?'
                },
                native: 'It was delicious! Could you please bring us the bill?'
            }
        ],
        objectives: [
            { id: 'request_table', description: 'Request a table and ask for the menu' },
            { id: 'ask_recommendation', description: 'Inquire about the chef\'s special or daily recommendation' },
            { id: 'order_courses', description: 'Order a beverage and your main dish' },
            { id: 'request_check', description: 'Ask for the bill and settle payment' }
        ]
    },
    {
        id: 'market_negotiation',
        title: 'Open-Air Market',
        personaName: 'Marco / Hassan / Sun-hee',
        role: 'Lively Market Merchant',
        category: 'shopping',
        difficulty: 'A2',
        vrmModel: 'man.vrm',
        icon: '🏷️',
        color: 'from-emerald-500/20 to-teal-500/10 border-emerald-500/30 text-emerald-300',
        badgeColor: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20',
        setting: 'A bustling outdoor bazaar with colorful stalls overflowing with fresh fruits, aromatic spices, and handmade wares.',
        summary: 'Inquire about fresh goods, haggle over prices, negotiate discounts for buying multiple items, and close a great deal.',
        scenarioPrompt: `You are Marco (or Hassan / Sun-hee), an expressive, energetic merchant running a popular stall at an open-air street market.
The user is a customer looking at your fresh fruit, regional delicacies, or handmade artisan goods.
Your responsibilities:
- Call out cheerfully to welcome the customer to your stall.
- Proudly describe the quality of your goods (e.g. freshly picked sweet oranges, artisan leather, rare spices).
- State an initial price that is slightly elevated to leave room for friendly negotiation.
- If the user asks for a lower price or offers a counter-price, negotiate with good humor: push back playfully, suggest buying two for a discount, or offer an extra small bonus item.
- Agree when a fair bargain is struck, wrap up the items, and thank them warmly for doing business.`,
        initialGreeting: {
            french: 'Approchez, approchez ! Goûtez ces fraises et ces oranges sucrées du verger ! Qu\'est-ce qui vous ferait envie aujourd\'hui ?',
            korean: '골라골라! 오늘 아침에 막 따온 신선하고 달콤한 과일 구경하고 가세요! 어떤 게 마음에 드세요?',
            spanish: '¡Pase, pase! Mire qué frutas tan frescas y ricas tenemos hoy. ¿Qué le gustaría llevar?',
            german: 'Kommen Sie näher! Schauen Sie sich diese frischen Erdbeeren und Orangen an. Was darf es heute sein?',
            japanese: 'いらっしゃい、いらっしゃい！朝採れたての新鮮な果物だよ！どれにする？',
            russian: 'Подходите, выбирайте! Самые свежие фрукты и специи на рынке! Что вас интересует?',
            ukrainian: 'Підходьте, вибирайте! Найсвіжіші фрукти та прянощі на базарі! Що вас цікавить?',
            english: 'Step right up! Come taste the freshest fruits and goods in the market! What catches your eye today?'
        },
        initialGreetingNative: 'Step right up! Come taste the freshest fruits and goods in the market! What catches your eye today?',
        suggestedPhrases: [
            {
                target: {
                    french: 'Combien coûte un kilo de ces belles oranges ?',
                    korean: '이 맛있는 오렌지 1kg에 얼마예요?',
                    spanish: '¿Cuánto cuesta el kilo de estas ricas naranjas?',
                    german: 'Wie viel kostet ein Kilo dieser Orangen?',
                    japanese: 'この美味しそうなオレンジは1キロいくらですか？',
                    russian: 'Сколько стоит килограмм этих апельсинов?',
                    ukrainian: 'Скільки коштує кілограм цих апельсинів?',
                    english: 'How much is a kilo of these oranges?'
                },
                native: 'How much is a kilo of these oranges?'
            },
            {
                target: {
                    french: 'C\'est un peu cher... Vous me faites un prix si j\'en prends deux kilos ?',
                    korean: '조금 비싼 것 같아요... 2kg 사면 좀 깎아주실 수 있나요?',
                    spanish: 'Está un poco caro... ¿Me hace un descuento si me llevo dos kilos?',
                    german: 'Das ist etwas teuer... Geben Sie mir einen Rabatt, wenn ich zwei Kilo nehme?',
                    japanese: '少し高いですね…2キロ買ったら少しまけてくれますか？',
                    russian: 'Это немного дорого... Сделаете скидку, если я возьму два килограмма?',
                    ukrainian: 'Це трохи дорого... Зробите знижку, якщо я візьму два кілограми?',
                    english: 'It\'s a bit expensive... Could you give me a discount if I take two kilos?'
                },
                native: 'It\'s a bit expensive... Could you give me a discount if I take two kilos?'
            },
            {
                target: {
                    french: 'D\'accord, c\'est une affaire ! Emballez-le moi, s\'il vous plaît.',
                    korean: '좋아요, 그렇게 해요! 예쁘게 담아주세요.',
                    spanish: '¡Trato hecho! Por favor, envuélvamelo.',
                    german: 'Abgemacht! Packen Sie es mir bitte ein.',
                    japanese: 'よし、その値段で買いましょう！包んでください。',
                    russian: 'Договорились, по рукам! Заверните мне, пожалуйста.',
                    ukrainian: 'Домовилися, по руках! Загорніть мені, будь ласка.',
                    english: 'Deal, that\'s a bargain! Please pack it up for me.'
                },
                native: 'Deal, that\'s a bargain! Please pack it up for me.'
            }
        ],
        objectives: [
            { id: 'inquire_price', description: 'Inquire about the price of goods or produce per kilo' },
            { id: 'bargain_discount', description: 'Counter-offer or propose a discount for buying in bulk' },
            { id: 'reach_agreement', description: 'Reach a mutually agreeable negotiated price' },
            { id: 'conclude_deal', description: 'Pay the merchant, thank them, and collect your items' }
        ]
    },
    {
        id: 'hotel_checkin',
        title: 'Boutique Hotel Lobby',
        personaName: 'Sofia / Kenji',
        role: 'Hotel Front Desk Concierge',
        category: 'travel',
        difficulty: 'A2',
        vrmModel: 'avatar.vrm',
        icon: '🏨',
        color: 'from-sky-500/20 to-blue-500/10 border-sky-500/30 text-sky-300',
        badgeColor: 'bg-sky-500/10 text-sky-400 border-sky-500/20',
        setting: 'An elegant hotel reception lobby with marble floors and soft background music.',
        summary: 'Check into your hotel room, ask about amenities and breakfast times, request room items, and ask for city tips.',
        scenarioPrompt: `You are Sofia (or Kenji), a friendly and polished front desk receptionist at a boutique hotel.
The user is a guest arriving to check in.
Your responsibilities:
- Greet the traveler warmly and ask for their reservation name.
- Confirm their booking details (length of stay, room type, breakfast included).
- Explain essential amenities: Wi-Fi network/password, breakfast serving hours and floor, and elevator location.
- Answer questions politely regarding luggage, late checkout, or nearby sights.
- Hand them their room key cards and wish them a wonderful stay.`,
        initialGreeting: {
            french: 'Bonjour et bienvenue à l\'Hôtel Étoile ! Vous venez vous enregistrer pour un séjour parmi nous ?',
            korean: '안녕하세요! 파를레 호텔에 오신 것을 환영합니다. 체크인 도와드릴까요? 예약자 분 성함이 어떻게 되시나요?',
            spanish: '¡Buenas tardes y bienvenido a nuestro hotel! ¿Viene a hacer el check-in?',
            german: 'Guten Tag und herzlich willkommen im Hotel! Möchten Sie einchecken?',
            japanese: 'いらっしゃいませ！ホテルへようこそ。チェックインのご案内をいたします。お名前をいただけますでしょうか？',
            russian: 'Здравствуйте! Добро пожаловать в наш отель. Вы на регистрацию заезда?',
            ukrainian: 'Доброго дня! Ласкаво просимо до нашого готелю. Ви на реєстрацію заїзду?',
            english: 'Hello and welcome to the hotel! Are you checking in today?'
        },
        initialGreetingNative: 'Hello and welcome to the hotel! Are you checking in today?',
        suggestedPhrases: [
            {
                target: {
                    french: 'Bonjour, j\'ai une réservation pour deux nuits au nom de Smith.',
                    korean: '안녕하세요, 2박 예약했습니다. 이름은 스미스입니다.',
                    spanish: 'Hola, tengo una reserva para dos noches a nombre de Smith.',
                    german: 'Guten Tag, ich habe eine Reservierung für zwei Nächte auf den Namen Smith.',
                    japanese: 'こんにちは、スミスの名前で2泊予約しています。',
                    russian: 'Здравствуйте, у меня бронь на две ночи на имя Смит.',
                    ukrainian: 'Доброго дня, у мене бронювання на дві ночі на ім\'я Сміт.',
                    english: 'Hello, I have a reservation for two nights under the name Smith.'
                },
                native: 'Hello, I have a reservation for two nights under the name Smith.'
            },
            {
                target: {
                    french: 'À quelle heure et où est servi le petit-déjeuner demain matin ?',
                    korean: '내일 아침 조식은 몇 시에 어디서 이용할 수 있나요?',
                    spanish: '¿A qué hora y dónde se sirve el desayuno mañana?',
                    german: 'Um wie viel Uhr und wo wird das Frühstück serviert?',
                    japanese: '明日の朝食は何時からどこでいただけますか？',
                    russian: 'Во сколько и где подается завтрак утром?',
                    ukrainian: 'О котрій годині та де подається сніданок вранці?',
                    english: 'What time and where is breakfast served tomorrow morning?'
                },
                native: 'What time and where is breakfast served tomorrow morning?'
            },
            {
                target: {
                    french: 'Pourriez-vous me donner le mot de passe du Wi-Fi ?',
                    korean: '와이파이 비밀번호 좀 알려주시겠어요?',
                    spanish: '¿Podría darme la contraseña del Wi-Fi, por favor?',
                    german: 'Könnten Sie mir bitte das WLAN-Passwort geben?',
                    japanese: 'Wi-Fiのパスワードを教えていただけますか？',
                    russian: 'Подскажите, пожалуйста, пароль от Wi-Fi?',
                    ukrainian: 'Підкажіть, будь ласка, пароль від Wi-Fi?',
                    english: 'Could you please give me the Wi-Fi password?'
                },
                native: 'Could you please give me the Wi-Fi password?'
            }
        ],
        objectives: [
            { id: 'state_reservation', description: 'State your name and reservation details' },
            { id: 'ask_breakfast', description: 'Ask about breakfast time and location' },
            { id: 'request_wifi', description: 'Ask for the Wi-Fi password and room key' },
            { id: 'ask_local_tip', description: 'Ask for a nearby attraction or restaurant recommendation' }
        ]
    },
    {
        id: 'asking_directions',
        title: 'City Center Crossroad',
        personaName: 'Mateo / Hana',
        role: 'Friendly Local Resident',
        category: 'travel',
        difficulty: 'A1',
        vrmModel: 'avatar.vrm',
        icon: '🗺️',
        color: 'from-teal-500/20 to-emerald-500/10 border-teal-500/30 text-teal-300',
        badgeColor: 'bg-teal-500/10 text-teal-400 border-teal-500/20',
        setting: 'A busy downtown intersection near a metro station and historic plaza.',
        summary: 'Ask a friendly local how to reach landmarks, metro lines, or museums, and understand directional cues.',
        scenarioPrompt: `You are Mateo (or Hana), a friendly, patient local resident walking down a central boulevard.
The user is a visitor who approaches you to ask for directions.
Your responsibilities:
- Greet them warmly and attentively.
- Give clear, step-by-step directions using cardinal markers (go straight for two blocks, turn left at the pharmacy, it is right next to the park).
- Mention approximately how long the walk takes or if it's better to take the bus or metro.
- Check if they understand or offer to clarify.
- Wish them a great time exploring your city.`,
        initialGreeting: {
            french: 'Bonjour ! Vous semblez chercher votre chemin. Est-ce que je peux vous aider ?',
            korean: '안녕하세요! 길을 찾고 계신가요? 혹시 도와드릴까요?',
            spanish: '¡Hola! Parece que estás buscando una dirección. ¿Te puedo ayudar?',
            german: 'Hallo! Suchen Sie den Weg? Kann ich Ihnen helfen?',
            japanese: 'こんにちは！道に迷われましたか？何かお手伝いしましょうか？',
            russian: 'Здравствуйте! Кажется, вы ищете дорогу. Чем я могу вам помочь?',
            ukrainian: 'Доброго дня! Здається, ви шукаєте дорогу. Чи можу я вам допомогти?',
            english: 'Hello! You look like you\'re searching for directions. Can I help you?'
        },
        initialGreetingNative: 'Hello! You look like you\'re searching for directions. Can I help you?',
        suggestedPhrases: [
            {
                target: {
                    french: 'Pardon, où se trouve la station de métro la plus proche ?',
                    korean: '실례지만, 가장 가까운 지하철역이 어디에 있나요?',
                    spanish: 'Disculpe, ¿dónde está la estación de metro más cercana?',
                    german: 'Entschuldigung, wo ist die nächste U-Bahn-Station?',
                    japanese: 'すみません、一番近い地下鉄の駅はどこですか？',
                    russian: 'Извините, где находится ближайшая станция метро?',
                    ukrainian: 'Перепрошую, де найближча станція метро?',
                    english: 'Excuse me, where is the nearest subway station?'
                },
                native: 'Excuse me, where is the nearest subway station?'
            },
            {
                target: {
                    french: 'Est-ce que c\'est loin à pied d\'ici ?',
                    korean: '여기서 걸어서 갈 만큼 가까운가요?',
                    spanish: '¿Está lejos caminando desde aquí?',
                    german: 'Ist es weit zu Fuß von hier?',
                    japanese: 'ここから歩いて行ける距離ですか？',
                    russian: 'Это далеко пешком отсюда?',
                    ukrainian: 'Це далеко пішки звідси?',
                    english: 'Is it far on foot from here?'
                },
                native: 'Is it far on foot from here?'
            },
            {
                target: {
                    french: 'Merci beaucoup pour votre aide ! Bonne journée !',
                    korean: '친절하게 알려주셔서 정말 감사합니다! 좋은 하루 보내세요!',
                    spanish: '¡Muchas gracias por su ayuda! ¡Que tenga buen día!',
                    german: 'Vielen Dank für Ihre Hilfe! Einen schönen Tag noch!',
                    japanese: '教えていただきありがとうございます！良い一日を！',
                    russian: 'Большое спасибо за помощь! Хорошего вам дня!',
                    ukrainian: 'Дуже дякую за допомогу! Гарного вам дня!',
                    english: 'Thank you so much for your help! Have a great day!'
                },
                native: 'Thank you so much for your help! Have a great day!'
            }
        ],
        objectives: [
            { id: 'ask_landmark', description: 'Politely excuse yourself and ask for directions to a destination' },
            { id: 'clarify_distance', description: 'Ask whether to walk or take public transportation' },
            { id: 'confirm_directions', description: 'Repeat or confirm the route (left, right, straight)' },
            { id: 'thank_local', description: 'Express gratitude and wish them a great day' }
        ]
    },
    {
        id: 'pharmacy_visit',
        title: 'Neighborhood Pharmacy',
        personaName: 'Dr. Clara / David',
        role: 'Caring Neighborhood Pharmacist',
        category: 'services',
        difficulty: 'B1',
        vrmModel: 'avatar.vrm',
        icon: '💊',
        color: 'from-purple-500/20 to-violet-500/10 border-purple-500/30 text-purple-300',
        badgeColor: 'bg-purple-500/10 text-purple-400 border-purple-500/20',
        setting: 'A clean, well-lit pharmacy with aisles of remedies, vitamins, and a prescription consultation counter.',
        summary: 'Describe common symptoms (headaches, fever, cold, stomachache), ask for recommendations, and understand dosage instructions.',
        scenarioPrompt: `You are Dr. Clara (or David), a kind, attentive community pharmacist.
The user is a customer seeking relief for an ailment.
Your responsibilities:
- Greet the customer and ask how you can help them feel better today.
- Listen carefully to their symptoms and ask follow-up questions (e.g. how long have they had it, do they have a fever or allergies).
- Recommend an appropriate over-the-counter remedy (pain relief, cough syrup, digestive aid).
- Explain dosage and instructions clearly (e.g. take one tablet twice a day with water after meals).
- Mention any precautions (e.g. do not drive, drink plenty of water) and wish them a fast recovery.`,
        initialGreeting: {
            french: 'Bonjour ! Comment puis-je vous aider aujourd\'hui ? Vous ne vous sentez pas bien ?',
            korean: '안녕하세요! 어디가 불편해서 오셨나요? 증상을 말씀해 주시면 알맞은 약을 찾아드릴게요.',
            spanish: '¡Hola! ¿En qué le puedo ayudar hoy? ¿No se siente bien?',
            german: 'Guten Tag! Wie kann ich Ihnen helfen? Fühlen Sie sich unwohl?',
            japanese: 'こんにちは！どのようなご症状でしょうか？お薬をお探しですか？',
            russian: 'Здравствуйте! Что вас беспокоит? Расскажите о ваших симптомах.',
            ukrainian: 'Доброго дня! Що вас турбує? Розкажіть про ваші симптоми.',
            english: 'Hello! How can I help you today? Are you not feeling well?'
        },
        initialGreetingNative: 'Hello! How can I help you today? Are you not feeling well?',
        suggestedPhrases: [
            {
                target: {
                    french: 'J\'ai un gros mal de tête et un peu de fièvre depuis ce matin.',
                    korean: '오늘 아침부터 두통이 심하고 열이 조금 나요.',
                    spanish: 'Tengo un fuerte dolor de cabeza y un poco de fiebre desde la mañana.',
                    german: 'Ich habe seit heute Morgen starke Kopfschmerzen und etwas Fieber.',
                    japanese: '今朝からひどい頭痛と微熱があります。',
                    russian: 'С сегодняшнего утра у меня сильная головная боль и небольшая температура.',
                    ukrainian: 'З сьогоднішнього ранку у мене сильний головний біль і невелика температура.',
                    english: 'I have a severe headache and a mild fever since this morning.'
                },
                native: 'I have a severe headache and a mild fever since this morning.'
            },
            {
                target: {
                    french: 'Quelle est la posologie ? Combien de fois par jour dois-je le prendre ?',
                    korean: '복용법이 어떻게 되나요? 하루에 몇 번 먹어야 하나요?',
                    spanish: '¿Cuál es la dosis? ¿Cuántas veces al día debo tomarlo?',
                    german: 'Wie ist die Dosierung? Wie oft am Tag soll ich es einnehmen?',
                    japanese: '用法・用量はどうですか？1日に何回飲めばいいですか？',
                    russian: 'Какая дозировка? Сколько раз в день нужно принимать?',
                    ukrainian: 'Яке дозування? Скільки разів на день потрібно приймати?',
                    english: 'What is the dosage? How many times a day should I take it?'
                },
                native: 'What is the dosage? How many times a day should I take it?'
            },
            {
                target: {
                    french: 'Merci docteur pour vos conseils et votre gentillesse !',
                    korean: '친절하게 설명해주셔서 감사합니다! 약 잘 챙겨 먹을게요.',
                    spanish: '¡Muchas gracias por su consejo y amabilidad!',
                    german: 'Vielen Dank für Ihren Rat und Ihre Hilfe!',
                    japanese: 'アドバイスありがとうございます！助かりました。',
                    russian: 'Большое спасибо за консультацию и помощь!',
                    ukrainian: 'Дуже дякую за консультацію та допомогу!',
                    english: 'Thank you for your advice and kindness!'
                },
                native: 'Thank you for your advice and kindness!'
            }
        ],
        objectives: [
            { id: 'describe_symptoms', description: 'Describe your symptoms and when they started' },
            { id: 'ask_medication', description: 'Ask for a recommended medicine or relief' },
            { id: 'understand_dosage', description: 'Ask how often and when to take the medicine' },
            { id: 'complete_consult', description: 'Thank the pharmacist for their guidance' }
        ]
    }
];

export function getScenarioById(id: string): RoleplayPersona | undefined {
    return ROLEPLAY_SCENARIOS.find(s => s.id === id);
}

export function getScenarioInitialGreeting(scenario: RoleplayPersona, targetLanguage: string): { target: string; native: string } {
    const langKey = (targetLanguage || 'english').toLowerCase().trim();
    const target = scenario.initialGreeting[langKey] || 
                   scenario.initialGreeting['english'] || 
                   Object.values(scenario.initialGreeting)[0] || 
                   'Hello!';
    return {
        target,
        native: scenario.initialGreetingNative
    };
}

export function getScenarioStarterPhrases(scenario: RoleplayPersona, targetLanguage: string): { target: string; native: string }[] {
    const langKey = (targetLanguage || 'english').toLowerCase().trim();
    return scenario.suggestedPhrases.map(p => ({
        target: p.target[langKey] || p.target['english'] || p.native,
        native: p.native
    }));
}
