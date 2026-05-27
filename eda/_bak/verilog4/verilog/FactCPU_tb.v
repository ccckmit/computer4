`include "FactCPU.v"

module FactCPU_tb;
    reg clk, rst;
    wire [15:0] result;
    wire done;

    FactCPU dut(.clk(clk), .rst(rst), .result(result), .done(done));

    initial begin
        $display("=== FactCPU (5!) Testbench ===");

        rst = 1;
        #10;
        rst = 0;
        #10;

        $display("5! = %d", result);
        if (result == 120)
            $display("PASS");
        else
            $display("FAIL: expected 120, got %d", result);

        $finish;
    end
endmodule
