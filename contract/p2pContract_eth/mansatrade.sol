// SPDX-License-Identifier: MIT

pragma solidity 0.8.17;

interface IERC20 {
    function totalSupply() external view returns (uint);
    function balanceOf(address account) external view returns (uint);
    function transfer(address recipient, uint amount) external returns (bool);
    function allowance(address owner, address spender) external view returns (uint);
    function approve(address spender, uint amount) external returns (bool);
    function transferFrom(address sender, address recipient, uint amount) external returns (bool);
    event Transfer(address indexed from, address indexed to, uint value);
    event Approval(address indexed owner, address indexed spender, uint value);
}

library SafeERC20 {
    using SafeMath for uint;

    function safeTransfer(IERC20 token, address to, uint value) internal {
        callOptionalReturn(token, abi.encodeWithSelector(token.transfer.selector, to, value));
    }

    function safeTransferFrom(IERC20 token, address from, address to, uint value) internal {
        callOptionalReturn(token, abi.encodeWithSelector(token.transferFrom.selector, from, to, value));
    }

    function safeApprove(IERC20 token, address spender, uint value) internal {
        require((value == 0) || (token.allowance(address(this), spender) == 0),
            "SafeERC20: approve from non-zero to non-zero allowance"
        );
        callOptionalReturn(token, abi.encodeWithSelector(token.approve.selector, spender, value));
    }
    function callOptionalReturn(IERC20 token, bytes memory data) private {
        require(isContract(address(token)), "SafeERC20: call to non-contract");

        // solhint-disable-next-line avoid-low-level-calls
        (bool success, bytes memory returndata) = address(token).call(data);
        require(success, "SafeERC20: low-level call failed");

        if (returndata.length > 0) { // Return data is optional
            // solhint-disable-next-line max-line-length
            require(abi.decode(returndata, (bool)), "SafeERC20: ERC20 operation did not succeed");
        }
    }

	function isContract(address addr) internal view returns (bool) {
        uint size;
        assembly { size := extcodesize(addr) }
        return size > 0;
    }
}

contract MansaTrade {
	using SafeMath for uint256;
    using SafeERC20 for IERC20;
    
	uint256 public fee = 45;
	uint256 public fir_fee = 80;
	uint256 public sec_fee = 20;
    address public admin = 0x95E7D2F2C071E1Cd8c10B8c9c579B007c67A37e1;
    address public owner;
    address public fir_div;
    address public sec_div;

	struct Offer {
        address owner;
        address token_address;
        string fiat;
        string rate;
        string payment_options;
        string public_key;
        string offer_terms;
        uint256 token_amount;
        uint256 min_limit;
        uint256 max_limit;
        uint256 bought;
        uint256 created_at;
        uint256 offer_index;
        uint8 time_limit;
        bool status;
        bool eth;
	}

	Offer[] internal offers;

    struct Order {
        address seller;
        string payment_option;
        string account_name;
        string account_mail;
        string receive_amount;
        uint8 status;
        bool buyer_confirm;
        bool seller_confirm;
        bool feedback;
        uint256 order_index;
        uint256 offer_index;
        uint256 sell_amount;
        uint256 created_at;
    }

    Order[] internal orders;

    struct User {
        bool verified;
        uint256 thumbs_up;
        uint256 thumbs_down;
        uint8 region;
        address user_address;
        uint256[] offer_indexes;
        uint256[] order_indexes;
    }

    mapping (address => User) internal users;

    modifier onlyOwner() {
        require(owner == msg.sender, "Ownable: caller is not the owner");
        _;
    }

    constructor (address _fir_div, address _sec_div) {
        owner = msg.sender;
        fir_div = _fir_div;
        sec_div = _sec_div;
    }
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    event CreateOrder(uint256 order_index);

    function updateFee(uint256 _fee, uint256 _fir_fee, uint256 _sec_fee) public {
        require(msg.sender == admin, "You are not admin!");
        fee = _fee;
        fir_fee = _fir_fee;
        sec_fee = _sec_fee;
    }

    function updateFeeDivider(address _fir_div, address _sec_div) public {
        require(msg.sender == admin, "You are not admin!");
        fir_div = _fir_div;
        sec_div = _sec_div;
    }

    function transferOwnership (address newOwner) public onlyOwner {
        require(newOwner != address(0), "Ownable: new owner is the zero address");
        emit OwnershipTransferred(owner, newOwner);
        owner = newOwner;
    }

    function createOffer (
        address token_address,
        string memory fiat, 
        string memory rate, 
        string memory payment_options, 
        string memory public_key, 
        string memory offer_terms, 
        uint8 time_limit, 
        bool eth,
        uint256 token_amount, 
        uint256 min_limit, 
        uint256 max_limit
    ) public {
        User storage user = users[msg.sender];
        if (user.user_address != msg.sender) {
            user.verified = false;
            user.thumbs_up = 0;
            user.thumbs_down = 0;
            user.user_address = msg.sender;
        }
        uint256 offer_index = offers.length;
        offers.push(Offer(msg.sender, token_address, fiat, rate, payment_options, public_key, offer_terms, token_amount, min_limit, max_limit, 0, block.timestamp, offer_index, time_limit, true, eth));
        user.offer_indexes.push(offer_index);
    }

    function updateOffer (
        string memory _fiat,
        string memory _payment_options,
        string memory _offer_terms,
        uint8 _time_limit,
        uint256 offer_index,
        uint256 _token_amount,
        uint256 _min_limit,
        uint256 _max_limit
    ) public {
        require(msg.sender == offers[offer_index].owner, "You are not owner of offer.");
        offers[offer_index].fiat = _fiat;
        offers[offer_index].payment_options = _payment_options;
        offers[offer_index].offer_terms = _offer_terms;
        offers[offer_index].time_limit = _time_limit;
        offers[offer_index].token_amount = _token_amount;
        offers[offer_index].min_limit = _min_limit;
        offers[offer_index].max_limit = _max_limit;
    }

    function cancelOffer (uint256 offer_index) public {
        require(msg.sender == offers[offer_index].owner, "You are not owner of offer.");
        offers[offer_index].status = false;
    }

    function createOrder (
        string memory payment_option,
        string memory account_name,
        string memory account_mail,
        string memory receive_amount,
        uint256 offer_index,
        uint256 sell_amount
    ) public payable {
        User storage user = users[msg.sender];
        if (user.user_address != msg.sender) {
            user.verified = false;
            user.thumbs_up = 0;
            user.thumbs_down = 0;
            user.user_address = msg.sender;
        }
        if (offers[offer_index].eth) {
            require(msg.value == sell_amount , "Please send as sell amount");
        } else {
            address tokenAddr = offers[offer_index].token_address;
            IERC20 token = IERC20(tokenAddr);
            require(token.allowance(msg.sender, address(this)) >= sell_amount , "Please approve token as sell amount");
            token.safeTransferFrom(msg.sender, address(this), sell_amount);
        }
        uint256 order_index = orders.length;
        orders.push(Order(msg.sender, payment_option, account_name, account_mail, receive_amount, 0, false, false, false, order_index, offer_index, sell_amount, block.timestamp));
        user.order_indexes.push(order_index);
        User storage buyer = users[offers[offer_index].owner];
        buyer.order_indexes.push(order_index);

        emit CreateOrder(order_index);
    }

    function buyerConfirm (uint256 order_index) public {
        uint256 offer_index = orders[order_index].offer_index;
        require (msg.sender == offers[offer_index].owner, "You are not buyer of order.");
        orders[order_index].buyer_confirm = true;
    }

    function confirmOrder (uint256 order_index) public {
        uint256 offer_index = orders[order_index].offer_index;
        require (orders[order_index].status == 0, "Order is not avaliable now.");
        require (orders[order_index].buyer_confirm, "Buyer is not confirm order yet.");
        require (msg.sender == orders[order_index].seller || msg.sender == owner, "You are not seller of order or not admin.");
        uint256 sellAmount = orders[order_index].sell_amount;
        if (offers[offer_index].eth) {
            payable(msg.sender).transfer(sellAmount * (10000 - fee * 2) / 10000);
            payable(fir_div).transfer(sellAmount * fee * 2 * fir_fee / 1000000);
            payable(sec_div).transfer(sellAmount * fee * 2 * sec_fee / 1000000);
        } else {
            IERC20 token = IERC20(offers[offer_index].token_address);
            token.safeTransfer(offers[offer_index].owner, sellAmount * (10000 - fee * 2) / 10000);
            token.safeTransfer(fir_div, sellAmount * fee * 2 * fir_fee / 1000000);
            token.safeTransfer(sec_div, sellAmount * fee * 2 * fir_fee / 1000000);
        }
        offers[offer_index].token_amount -= sellAmount;
        offers[offer_index].bought += sellAmount;
        orders[order_index].seller_confirm = true;
        orders[order_index].status = 1;

        if (offers[offer_index].token_amount == 0) {
            offers[offer_index].status = true;
        }
    }

    function cancelOrder (uint256 order_index) public {
        uint256 offer_index = orders[order_index].offer_index;
        require (msg.sender == offers[offer_index].owner || msg.sender == orders[order_index].seller || msg.sender == owner, "You are not buyer or seller or admin.");
        require(orders[order_index].status != 1, "Order is completed.");
        uint256 sellAmount = orders[order_index].sell_amount;
        if (offers[offer_index].eth) {
            payable(orders[order_index].seller).transfer(sellAmount);
        } else {
            IERC20 token = IERC20(offers[offer_index].token_address);
            token.safeTransfer(orders[order_index].seller, sellAmount);
        }

        orders[order_index].status = 2;
    }

    function createUser () public {
        User storage user = users[msg.sender];
        user.verified = false;
        user.thumbs_up = 0;
        user.thumbs_down = 0;
        user.user_address = msg.sender;
    }

    function verifyUser(address _user) public {
        require(msg.sender == admin, "You are not admin!");
        User storage user = users[_user];
        user.verified = true;
    }

    function updateUser(uint8 _region) public {
        User storage user = users[msg.sender];
        user.region = _region;
    }

    function thumbUser (bool flag, address _user, uint256 order_index) public {
        require(msg.sender != _user, "You cannot claim yours");
        User storage user = users[_user];
        Order storage order = orders[order_index];
        order.feedback = true;
        if (flag) user.thumbs_up++;
        else user.thumbs_down++;
    }

    function getUser(address _address) public view returns (User memory) {
        return users[_address];
    }

    function getOffers() public view returns(Offer[] memory) { return offers; }

    function getOrders() public view returns(Order[] memory) { return orders; }

    function getOfferByIndex(uint256 index) public view returns (Offer memory) {
        return offers[index];
    }

    function getOfferIndexesOfUser(address userAddress) public view returns (uint256[] memory) {
        User storage user = users[userAddress];
        return user.offer_indexes;
    }


    function getOrderIndexesOfUser(address userAddress) public view returns (uint256[] memory) {
        User storage user = users[userAddress];
        return user.order_indexes;
    }

    function getOrderByIndex(uint256 index) public view returns (Order memory) {
        return orders[index];
    }
}



library SafeMath {

    function add(uint256 a, uint256 b) internal pure returns (uint256) {
        uint256 c = a + b;
        require(c >= a, "SafeMath: addition overflow");

        return c;
    }

    function sub(uint256 a, uint256 b) internal pure returns (uint256) {
        require(b <= a, "SafeMath: subtraction overflow");
        uint256 c = a - b;

        return c;
    }

    function mul(uint256 a, uint256 b) internal pure returns (uint256) {
        if (a == 0) {
            return 0;
        }

        uint256 c = a * b;
        require(c / a == b, "SafeMath: multiplication overflow");

        return c;
    }

    function div(uint256 a, uint256 b) internal pure returns (uint256) {
        require(b > 0, "SafeMath: division by zero");
        uint256 c = a / b;

        return c;
    }
}